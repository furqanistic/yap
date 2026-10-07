import "./Switch.css";
import { LiquidGlassSurface } from "@/components/liquid-glass";

interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
}

export function Switch({ checked, onChange, disabled, ...aria }: SwitchProps) {
  return (
    <LiquidGlassSurface
      as="button"
      type="button"
      preset="interactive"
      refraction={false}
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      className="switch"
      onClick={() => onChange(!checked)}
      {...aria}
    >
      <span className="switch__thumb" />
    </LiquidGlassSurface>
  );
}
