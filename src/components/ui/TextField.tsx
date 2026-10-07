import { IconMagnifer } from "@devigner-ui/icons/Magnifer";
import { forwardRef, type InputHTMLAttributes } from "react";
import "./Field.css";

interface TextFieldProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "value" | "onChange" | "type"> {
  value: string;
  onChange: (value: string) => void;
  /** `search` adds a leading magnifier icon. */
  variant?: "default" | "search";
}

export const TextField = forwardRef<HTMLInputElement, TextFieldProps>(function TextField(
  { value, onChange, variant = "default", className, ...rest },
  ref,
) {
  const search = variant === "search";

  return (
    <span className={["text-field", search && "text-field--search", className].filter(Boolean).join(" ")}>
      {search && <IconMagnifer className="text-field__icon" strokeWidth={1.75} />}
      <input
        ref={ref}
        type={search ? "search" : "text"}
        className="field field--text"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        spellCheck={false}
        autoComplete="off"
        {...rest}
      />
    </span>
  );
});
