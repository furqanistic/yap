import type { IconProps } from "@devigner-ui/icons";
import { IconSpinner } from "@devigner-ui/icons/Spinner";
import { forwardRef, type ButtonHTMLAttributes, type ComponentType } from "react";
import "./Button.css";

export type ButtonVariant = "primary" | "secondary" | "quiet" | "destructive";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: "default" | "small";
  /** Decorative icon shown before the label. */
  icon?: ComponentType<IconProps>;
  /** Shows a spinner and blocks clicks while an action runs. */
  loading?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { variant = "secondary", size = "default", icon: Icon, loading, disabled, className, children, ...rest },
  ref,
) {
  const classes = ["button", `button--${variant}`, size === "small" && "button--small", className]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      ref={ref}
      type="button"
      className={classes}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? (
        <IconSpinner className="button__icon button__spinner" strokeWidth={1.75} />
      ) : (
        Icon && <Icon className="button__icon" strokeWidth={1.75} />
      )}
      {children}
    </button>
  );
});
