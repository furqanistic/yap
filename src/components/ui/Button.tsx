import type { IconProps } from "@devigner-ui/icons";
import { type ButtonHTMLAttributes, type ComponentType, type ReactNode } from "react";
import "./Button.css";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children?: ReactNode;
  icon?: ComponentType<IconProps>;
  iconPosition?: "left" | "right";
}

export function Button({
  children,
  icon: Icon,
  iconPosition = "right",
  className = "",
  type = "button",
  ...props
}: ButtonProps) {
  return (
    <button type={type} className={`button ${className}`.trim()} {...props}>
      {Icon && iconPosition === "left" && <Icon className="button__icon" strokeWidth={1.75} />}
      {children && <span>{children}</span>}
      {Icon && iconPosition === "right" && <Icon className="button__icon" strokeWidth={1.75} />}
    </button>
  );
}
