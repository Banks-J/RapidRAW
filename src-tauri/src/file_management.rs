use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, GenericImageView, ImageBuffer, Luma};
use rayon::prelude::*;
use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::image_processing::{
    apply_crop, get_all_adjustments_from_json, GpuContext, ImageMetadata, Crop, apply_rotation,
};
use crate::mask_generation::{MaskDefinition, generate_mask_bitmap};
use crate::raw_processing;
use crate::{gpu_processing, AppState};

const THUMBNAIL_WIDTH: u32 = 720;

fn is_raw_file(path: &str) -> bool {
    let lower_path = path.to_lowercase();
    matches!(
        lower_path.split('.').last(),
        Some("arw") | Some("cr2") | Some("cr3") | Some("nef") | Some("dng") |
        Some("raf") | Some("orf") | Some("pef") | Some("rw2")
    )
}

#[derive(Serialize, Debug, Clone)]
pub struct ImageFile {
    path: String,
    modified: u64,
}

#[tauri::command]
pub fn list_images_in_dir(path: String) -> Result<Vec<ImageFile>, String> {
    let entries = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            !path
                .file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with('.'))
        })
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|s| s.to_str())
                .map_or(false, |ext_lower| {
                    let ext = ext_lower.to_lowercase();
                    ["jpg", "jpeg", "png", "gif", "bmp", "arw", "cr2", "cr3", "nef", "dng", "raf", "orf", "pef", "rw2"].contains(&ext.as_str())
                })
        })
        .map(|path| {
            let modified = fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            ImageFile {
                path: path.to_string_lossy().into_owned(),
                modified,
            }
        })
        .collect();
    Ok(entries)
}

#[derive(Serialize, Debug)]
pub struct FolderNode {
    pub name: String,
    pub path: String,
    pub children: Vec<FolderNode>,
}

fn scan_dir_recursive(path: &Path) -> Result<Vec<FolderNode>, std::io::Error> {
    let mut children = Vec::new();
    
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Could not scan directory '{}': {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    for entry in entries.filter_map(std::result::Result::ok) {
        let current_path = entry.path();
        let is_hidden = current_path
            .file_name()
            .and_then(|s| s.to_str())
            .map_or(false, |s| s.starts_with('.'));

        if current_path.is_dir() && !is_hidden {
            let sub_children = scan_dir_recursive(&current_path)?;
            let has_images = list_images_in_dir(current_path.to_string_lossy().into_owned())
                .map_or(false, |images| !images.is_empty());
            if !sub_children.is_empty() || has_images {
                children.push(FolderNode {
                    name: current_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    path: current_path.to_string_lossy().into_owned(),
                    children: sub_children,
                });
            }
        }
    }
    Ok(children)
}

fn get_folder_tree_sync(path: String) -> Result<FolderNode, String> {
    let root_path = Path::new(&path);
    let name = root_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let children = scan_dir_recursive(root_path).map_err(|e| e.to_string())?;
    Ok(FolderNode {
        name,
        path,
        children,
    })
}

#[tauri::command]
pub async fn get_folder_tree(path: String) -> Result<FolderNode, String> {
    match tauri::async_runtime::spawn_blocking(move || get_folder_tree_sync(path)).await {
        Ok(Ok(folder_node)) => Ok(folder_node),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(format!("Failed to execute folder tree task: {}", e)),
    }
}

pub fn get_sidecar_path(image_path: &str) -> PathBuf {
    let path = PathBuf::from(image_path);
    let original_filename = path.file_name().unwrap_or_default().to_string_lossy();
    let new_filename = format!("{}.rrdata", original_filename);
    path.with_file_name(new_filename)
}

fn generate_thumbnail_data(
    path_str: &str,
    gpu_context: Option<&GpuContext>,
) -> Result<DynamicImage> {
    let file_bytes = fs::read(path_str)?;

    let (base_image, original_dims): (DynamicImage, (u32, u32)) = if is_raw_file(path_str) {
        let raw_info = rawloader::decode(&mut Cursor::new(&file_bytes))?;
        let crops = raw_info.crops;
        let full_width = raw_info.width as u32 - crops[3] as u32 - crops[1] as u32;
        let full_height = raw_info.height as u32 - crops[0] as u32 - crops[2] as u32;
        (
            raw_processing::develop_raw_thumbnail(&file_bytes)?,
            (full_width, full_height),
        )
    } else {
        let img = image::load_from_memory(&file_bytes)?;
        let dims = img.dimensions();
        (img, dims)
    };

    let sidecar_path = get_sidecar_path(path_str);
    let metadata: Option<ImageMetadata> = fs::read_to_string(sidecar_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());

    if let (Some(context), Some(meta)) = (gpu_context, metadata) {
        if !meta.adjustments.is_null() {
            const THUMBNAIL_PROCESSING_DIM: u32 = 1280;
            let (full_w, full_h) = original_dims;

            let (processing_base, scale_for_gpu) = 
                if full_w > THUMBNAIL_PROCESSING_DIM || full_h > THUMBNAIL_PROCESSING_DIM {
                    let base = if is_raw_file(path_str) { 
                        base_image 
                    } else { 
                        base_image.thumbnail(THUMBNAIL_PROCESSING_DIM, THUMBNAIL_PROCESSING_DIM) 
                    };
                    let scale = if full_w > 0 { base.width() as f32 / full_w as f32 } else { 1.0 };
                    (base, scale)
                } else {
                    (base_image, 1.0)
                };

            let rotation_degrees = meta.adjustments["rotation"].as_f64().unwrap_or(0.0) as f32;
            let rotated_image = apply_rotation(&processing_base, rotation_degrees);

            let crop_data: Option<Crop> = serde_json::from_value(meta.adjustments["crop"].clone()).ok();
            let scaled_crop_json = if let Some(c) = &crop_data {
                serde_json::to_value(Crop {
                    x: c.x * scale_for_gpu as f64,
                    y: c.y * scale_for_gpu as f64,
                    width: c.width * scale_for_gpu as f64,
                    height: c.height * scale_for_gpu as f64,
                }).unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            };

            let cropped_preview = apply_crop(rotated_image, &scaled_crop_json);
            let (preview_w, preview_h) = cropped_preview.dimensions();
            
            let unscaled_crop_offset = crop_data.map_or((0.0, 0.0), |c| (c.x as f32, c.y as f32));
            
            let mask_definitions: Vec<MaskDefinition> = meta.adjustments.get("masks")
                .and_then(|m| serde_json::from_value(m.clone()).ok())
                .unwrap_or_else(Vec::new);

            let mask_bitmaps: Vec<ImageBuffer<Luma<u8>, Vec<u8>>> = mask_definitions.iter()
                .filter_map(|def| generate_mask_bitmap(def, preview_w, preview_h, scale_for_gpu, (unscaled_crop_offset.0 * scale_for_gpu, unscaled_crop_offset.1 * scale_for_gpu)))
                .collect();

            let gpu_adjustments = get_all_adjustments_from_json(&meta.adjustments);

            if let Ok(processed_image) = gpu_processing::process_and_get_dynamic_image(context, &cropped_preview, gpu_adjustments, &mask_bitmaps) {
                return Ok(processed_image);
            } else {
                return Ok(cropped_preview);
            }
        }
    }

    Ok(base_image)
}


fn encode_thumbnail(image: &DynamicImage) -> Result<Vec<u8>> {
    let thumbnail = image.thumbnail(THUMBNAIL_WIDTH, THUMBNAIL_WIDTH);
    let mut buf = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buf, 75);
    encoder.encode_image(&thumbnail.to_rgba8())?;
    Ok(buf.into_inner())
}

#[tauri::command]
pub fn generate_thumbnails(
    paths: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<HashMap<String, String>, String> {
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let thumb_cache_dir = cache_dir.join("thumbnails");
    if !thumb_cache_dir.exists() {
        fs::create_dir_all(&thumb_cache_dir).map_err(|e| e.to_string())?;
    }

    let state = app_handle.state::<AppState>();
    let gpu_context = gpu_processing::get_or_init_gpu_context(&state).ok();

    let thumbnails: HashMap<String, String> = paths
        .par_iter()
        .filter_map(|path_str| {
            let original_path = Path::new(path_str);
            let sidecar_path = get_sidecar_path(path_str);

            let img_mod_time = fs::metadata(original_path)
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            let sidecar_mod_time = fs::metadata(&sidecar_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
                .unwrap_or(0);

            let mut hasher = blake3::Hasher::new();
            hasher.update(path_str.as_bytes());
            hasher.update(&img_mod_time.to_le_bytes());
            hasher.update(&sidecar_mod_time.to_le_bytes());
            let hash = hasher.finalize();
            let cache_filename = format!("{}.jpg", hash.to_hex());
            let cache_path = thumb_cache_dir.join(cache_filename);

            if cache_path.exists() {
                if let Ok(data) = fs::read(&cache_path) {
                    let base64_str = general_purpose::STANDARD.encode(&data);
                    return Some((
                        path_str.clone(),
                        format!("data:image/jpeg;base64,{}", base64_str),
                    ));
                }
            }

            if let Ok(thumb_image) = generate_thumbnail_data(path_str, gpu_context.as_ref()) {
                if let Ok(thumb_data) = encode_thumbnail(&thumb_image) {
                    let _ = fs::write(&cache_path, &thumb_data);
                    let base64_str = general_purpose::STANDARD.encode(&thumb_data);
                    return Some((
                        path_str.clone(),
                        format!("data:image/jpeg;base64,{}", base64_str),
                    ));
                }
            }
            None
        })
        .collect();

    Ok(thumbnails)
}

#[tauri::command]
pub fn generate_thumbnails_progressive(
    paths: Vec<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let cache_dir = app_handle
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let thumb_cache_dir = cache_dir.join("thumbnails");
    if !thumb_cache_dir.exists() {
        fs::create_dir_all(&thumb_cache_dir).map_err(|e| e.to_string())?;
    }

    let app_handle_clone = app_handle.clone();
    let total_count = paths.len();
    let completed_count = Arc::new(AtomicUsize::new(0));

    thread::spawn(move || {
        let state = app_handle.state::<AppState>();
        let gpu_context = gpu_processing::get_or_init_gpu_context(&state).ok();

        paths.par_iter().for_each(|path_str| {
            let result = (|| -> Option<(String, u8)> {
                let original_path = Path::new(path_str);
                let sidecar_path = get_sidecar_path(path_str);

                let img_mod_time = fs::metadata(original_path)
                    .ok()?
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();

                let (sidecar_mod_time, rating) = if let Ok(content) = fs::read_to_string(&sidecar_path) {
                    let mod_time = fs::metadata(&sidecar_path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let rating_val = serde_json::from_str::<ImageMetadata>(&content)
                        .ok()
                        .map(|m| m.rating)
                        .unwrap_or(0);
                    (mod_time, rating_val)
                } else {
                    (0, 0)
                };

                let mut hasher = blake3::Hasher::new();
                hasher.update(path_str.as_bytes());
                hasher.update(&img_mod_time.to_le_bytes());
                hasher.update(&sidecar_mod_time.to_le_bytes());
                let hash = hasher.finalize();
                let cache_filename = format!("{}.jpg", hash.to_hex());
                let cache_path = thumb_cache_dir.join(cache_filename);

                if cache_path.exists() {
                    if let Ok(data) = fs::read(&cache_path) {
                        let base64_str = general_purpose::STANDARD.encode(&data);
                        return Some((format!("data:image/jpeg;base64,{}", base64_str), rating));
                    }
                }

                if let Ok(thumb_image) = generate_thumbnail_data(path_str, gpu_context.as_ref()) {
                    if let Ok(thumb_data) = encode_thumbnail(&thumb_image) {
                        let _ = fs::write(&cache_path, &thumb_data);
                        let base64_str = general_purpose::STANDARD.encode(&thumb_data);
                        return Some((format!("data:image/jpeg;base64,{}", base64_str), rating));
                    }
                }
                None
            })();

            if let Some((thumbnail_data, rating)) = result {
                let _ = app_handle_clone.emit(
                    "thumbnail-generated",
                    serde_json::json!({ "path": path_str, "data": thumbnail_data, "rating": rating }),
                );
            }

            let completed = completed_count.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app_handle_clone.emit(
                "thumbnail-progress",
                serde_json::json!({ "completed": completed, "total": total_count }),
            );
        });

        let _ = app_handle_clone.emit("thumbnail-generation-complete", true);
    });

    Ok(())
}