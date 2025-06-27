<div align="center">

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/react-%2320232a.svg?style=for-the-badge&logo=react&logoColor=%2361DAFB)](https://react.dev/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![AGPL-3.0](https://img.shields.io/badge/License-AGPL_v3-blue.svg?style=for-the-badge)](https://opensource.org/licenses/AGPL-3.0)
[![GitHub stars](https://img.shields.io/github/stars/CyberTimon/RapidRAW?style=for-the-badge&logo=github&label=Stars)](https://github.com/CyberTimon/RapidRAW/stargazers)

</div>

<br>

<p align="center">
  <img src="https://raw.githubusercontent.com/CyberTimon/RapidRAW/main/github_assets/editor.png" alt="RapidRAW Editor">
</p>

# RapidRAW

> A blazingly-fast, non-destructive, and GPU-accelerated RAW image editor built with performance in mind.

RapidRAW is a feature-rich image manipulation program, similar to Adobe Lightroom, designed from the ground up for maximum performance. By leveraging a **Rust** backend with a **WGPU-based** processing pipeline and a modern **React** frontend inside a **Tauri** shell, it offers a responsive and efficient editing experience for photographers and enthusiasts. Thanks to Tauri, it's a lightweight, cross-platform application available for **Windows, macOS, and Linux**.

This project was developed by me, [Timon Käch](https://github.com/CyberTimon), as a personal challenge in just two weeks at the age of 18.

**Table of Contents**
- [Key Features](#key-features)
- [Demo & Screenshots](#demo--screenshots)
- [The Story](#the-story)
- [Development Timeline](#development-timeline)
- [Getting Started](#getting-started)
- [Contributing](#contributing)
- [Support the Project](#support-the-project)
- [License](#license)

---

## Key Features

<table width="100%">
  <tr>
    <td valign="top" width="50%">
      <h4>Core Editing Engine</h4>
      <ul>
        <li><strong>GPU-Accelerated Processing:</strong> All image adjustments are processed on the GPU using a custom WGSL shader for real-time feedback.</li>
        <li><strong>Full RAW Support:</strong> Leverages the <code>rawloader</code> crate to support a wide range of camera formats, featuring two advanced demosaicing algorithms (including Menon2007).</li>
        <li><strong>Non-Destructive Workflow:</strong> All edits are stored in a <code>.rrdata</code> sidecar file, leaving your original images untouched.</li>
        <li><strong>Advanced Masking:</strong> Local adjustments with Brush, Linear, and Radial masks. The mask system is bitmap-based for future extensibility.</li>
        <li><strong>32-bit Floating-Point Precision:</strong> Ensures high-quality adjustments without banding or data loss.</li>
      </ul>
      <h4>Professional Grade Adjustments</h4>
      <ul>
        <li><strong>Tonal Controls:</strong> Exposure, Contrast, Highlights, Shadows, Whites, and Blacks.</li>
        <li><strong>Tone Curves:</strong> Full control over Luma, Red, Green, and Blue channels.</li>
        <li><strong>Color Grading:</strong> Temperature, Tint, Vibrance, Saturation, and a full HSL (Hue, Saturation, Luminance) color mixer.</li>
        <li><strong>Detail Enhancement:</strong> Sharpening, Clarity, Structure, and Noise Reduction (Luminance & Color).</li>
        <li><strong>Effects:</strong> Dehaze, Vignette, and realistic Film Grain simulation.</li>
        <li><strong>Transform Tools:</strong> Crop with aspect ratio locking, Rotate, and Flip.</li>
      </ul>
    </td>
    <td valign="top" width="50%">
      <h4>Library & Workflow</h4>
      <ul>
        <li><strong>Complete Image Library:</strong> Sort, rate, and manage your photos efficiently.</li>
        <li><strong>Folder Management:</strong> Integrated folder tree, create, rename, and delete folders directly within the app.</li>
        <li><strong>File Operations:</strong> Copy, move, and duplicate images and their associated edits.</li>
        <li><strong>Filmstrip View:</strong> Quickly navigate between images in your current folder while editing.</li>
        <li><strong>Batch Operations:</strong> Apply adjustments or export multiple images at once.</li>
        <li><strong>EXIF Data Viewer:</strong> Inspect your camera's metadata.</li>
      </ul>
      <h4>Productivity & UI</h4>
      <ul>
        <li><strong>Preset System:</strong> Create, save, import, and export your favorite looks.</li>
        <li><strong>Copy & Paste Settings:</strong> Quickly transfer adjustments between images.</li>
        <li><strong>Undo/Redo History:</strong> A robust history system for every edit.</li>
        <li><strong>Customizable UI:</strong> Resizable panels and multiple beautiful UI themes with smooth animations.</li>
        <li><strong>Advanced Exporting:</strong> Control file format (JPEG, PNG, TIFF), quality, and resizing options on export.</li>
      </ul>
    </td>
  </tr>
</table>

## Demo & Screenshots

Here's RapidRAW in action.

<!--
  PLACEHOLDER: Replace these placeholder images with actual screenshots or GIFs of your application.
  This is the most important part to make your project look impressive!
-->
<p align="center">
  <img src="https://via.placeholder.com/800x450/1A1D1B/FFFFFF?text=Main+Editor+View" alt="Editor View" style="max-width: 100%;"><br>
  <em>The main editor interface with all adjustment panels.</em>
</p>
<br>
<table width="100%">
  <tr>
    <td width="50%" align="center">
      <img src="https://via.placeholder.com/400x225/1A1D1B/FFFFFF?text=Library+View" alt="Library View" style="max-width: 100%;"><br>
      <em>Full-featured image library.</em>
    </td>
    <td width="50%" align="center">
      <img src="https://via.placeholder.com/400x225/1A1D1B/FFFFFF?text=Masking+Tools" alt="Masking Tools" style="max-width: 100%;"><br>
      <em>Powerful local adjustments with masks.</em>
    </td>
  </tr>
</table>

## The Story

#### The Motivation
As a photography enthusiast, I often found existing software to be sluggish and resource-heavy on my machine. Born from the desire for a more responsive and streamlined photo editing experience, I set out to build my own. The goal was to create a tool that was not only fast but also helped me learn the intricate details of digital image processing and camera technology.

#### The Challenge
I gave myself a strict deadline of **two weeks** to go from an empty folder to a functional, feature-rich application. This personal challenge pushed me to learn quickly, make pragmatic decisions, and focus intensely on the core architecture and user experience.

#### The Process
The foundation is built on Rust for its safety and performance, and Tauri for its ability to create lightweight, cross-platform desktop apps with a web frontend. The entire image processing pipeline is offloaded to the GPU via WGPU and a custom WGSL shader, ensuring that even on complex edits with multiple masks, the UI remains fluid.

I am immensely grateful for Google's Gemini suite of AI models. As an 18-year-old without a formal background in advanced mathematics or image science, the AI Studio's free tier was an invaluable assistant, helping me research and implement complex concepts like the Menon demosaicing algorithm.

## Development Timeline

This project was built in an intense two-week sprint. Here's a summary of the progress:

<details>
<summary><strong>Click to expand the day-by-day development log</strong></summary>

*   **Day 1: June 13th, 2025** - Project inception, basic Tauri setup, and initial brightness/contrast shader implementation.
*   **Day 2: June 14th** - Core architecture refactor, full library support (folder tree, image list), and optimized image loading. Implemented histogram and curve editor support. Added UI themes.
*   **Day 3: June 15th** - Implemented a working crop tool, preset system, and context menus. Enabled auto-saving of edits to sidecar files and auto-thumbnail generation. Refined color adjustments.
*   **Day 4: June 16th** - Initial prototype for local adjustments with masking. Added mask support to presets. Bug-free image preview switching.
*   **Day 5: June 17th** - Major UI overhaul. Created the filmstrip and resizable panel layout. Fixed mask scaling issues and improved the library/welcome screen.
*   **Day 6: June 18th** - Performance tuning. Reduced GPU calls for adjustments, leading to a much smoother cropping and editing experience. Implemented saving of panel UI state.
*   **Day 7: June 19th** - Enhanced library functionality. Added multi-selection and the ability to copy/paste adjustments across multiple images.
*   **Day 8: June 20th** - Implemented initial RAW file support and an EXIF metadata viewer.
*   **Day 9: June 21st** - Added advanced detail adjustments (Clarity, Sharpening, Dehaze, etc.) and film grain. Developed a linear RAW processing pipeline.
*   **Day 10: June 22nd** - Implemented layer stacking for smooth preview transitions. Built a robust export panel with batch export capabilities. Added import/export for presets.
*   **Day 11: June 23rd** - Added full undo/redo functionality integrated with a custom history hook. Improved context menus and completed the settings panel.
*   **Day 12: June 24th** - Implemented image rotation and fixed all mask scaling/alignment issues related to cropping and rotation.
*   **Day 13: June 25th** - Rewrote the mask system to be bitmap-based. Implemented brush and linear gradient tools, with semi-transparent visualization.
*   **Day 14: June 26th-27th** - Final polish. Added universal keyboard shortcuts, full adjustment support for masks, theme management, and final UI/UX improvements.

</details>

## Getting Started

You have two options to run RapidRAW:

**1. Download the Latest Release (Recommended)**

Grab the pre-built installer or application bundle for your operating system from the [**Releases**](https://github.com/CyberTimon/RapidRAW/releases) page.

**2. Build from Source**

If you want to build the project yourself, you'll need to have [Rust](https://www.rust-lang.org/tools/install) and [Node.js](https://nodejs.org/) installed.

```bash
# 1. Clone the repository
git clone https://github.com/CyberTimon/RapidRAW.git
cd RapidRAW

# 2. Install frontend dependencies
npm install

# 3. Build and run the application
npm run tauri dev
```

## Contributing

Contributions are welcome and highly appreciated! Whether it's reporting a bug, suggesting a feature, or submitting a pull request, your help makes this project better. Please feel free to open an issue to discuss your ideas.

## Support the Project

As an 18-year-old developer balancing this project with an apprenticeship, your support means the world. If you find RapidRAW useful or exciting, please consider donating to help me dedicate more time to its development and cover any associated costs.

<!-- PLACEHOLDER: Add your Ko-fi or other donation links here -->
-   **Ko-fi:** [Donate on Ko-fi](https://ko-fi.com/your-username)
-   **Crypto:**
    -   BTC: `your_btc_address_here`
    -   ETH: `your_eth_address_here`

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPL-3.0)**. I chose this license to ensure that RapidRAW and any of its derivatives will always remain open-source and free for the community. It protects the project from being used in closed-source commercial software, ensuring that improvements benefit everyone.

See the [LICENSE](LICENSE) file for more details.