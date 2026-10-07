import { useEffect, useState } from "react";
import "./KeyboardKeys.css";

export interface KeyDef {
  label: string;
  sublabel?: string;
  size?: "sm" | "md" | "lg" | "xl";
  /** `KeyboardEvent.key` value that visually presses this key. */
  keyCode?: string;
}

interface KeyboardKeysProps {
  keys: KeyDef[];
  /** Small caption shown above the keys. */
  label?: string;
  "aria-label"?: string;
}

function Key({ label, sublabel, size = "md", keyCode }: KeyDef) {
  const [pressed, setPressed] = useState(false);

  useEffect(() => {
    if (!keyCode) return;
    const target = keyCode.toLowerCase();
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === target) setPressed(true);
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key.toLowerCase() === target) setPressed(false);
    };
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, [keyCode]);

  const classes = ["keyboard-key", `keyboard-key--${size}`, pressed ? "is-pressed" : ""]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      type="button"
      className={classes}
      tabIndex={-1}
      onMouseDown={() => setPressed(true)}
      onMouseUp={() => setPressed(false)}
      onMouseLeave={() => setPressed(false)}
      onTouchStart={() => setPressed(true)}
      onTouchEnd={() => setPressed(false)}
    >
      <span className="keyboard-key__shadow" />
      <span className="keyboard-key__surface">
        <span className="keyboard-key__label">
          {sublabel && <span className="keyboard-key__sublabel">{sublabel}</span>}
          <span className="keyboard-key__text">{label}</span>
        </span>
      </span>
    </button>
  );
}

/**
 * Physical-feeling keycaps that press down when the matching keys are held
 * on the real keyboard (or clicked). Used on the welcome screen to teach
 * the push-to-talk shortcut.
 */
export function KeyboardKeys({ keys, label, "aria-label": ariaLabel }: KeyboardKeysProps) {
  return (
    <div className="keyboard-keys" role="group" aria-label={ariaLabel ?? "Keyboard shortcut"}>
      {label && <p className="keyboard-keys__hint">{label}</p>}
      <div className="keyboard-keys__row">
        {keys.map((key) => (
          <Key key={`${key.label}-${key.keyCode ?? "click"}`} {...key} />
        ))}
      </div>
    </div>
  );
}
