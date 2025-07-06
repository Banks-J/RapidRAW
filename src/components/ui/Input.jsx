import React from 'react';
import clsx from 'clsx';

/**
 * A reusable text input component that matches the application's design system.
 * It uses theme variables for colors and consistent styling for focus and disabled states.
 *
 * @param {string} className - Additional classes to apply to the input.
 * @param {string} type - The type of the input (e.g., 'text', 'password', 'email').
 * @param {object} props - Other standard input props (value, onChange, placeholder, etc.).
 */
const Input = React.forwardRef(({ className, type = 'text', ...props }, ref) => {
  return (
    <input
      type={type}
      className={clsx(
        'flex h-10 w-full rounded-md border px-3 py-2 text-sm',
        'bg-bg-primary border-border-color text-text-primary placeholder:text-text-secondary',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'file:border-0 file:bg-transparent file:text-sm file:font-medium',
        className
      )}
      ref={ref}
      {...props}
    />
  );
});

Input.displayName = 'Input';

export default Input;