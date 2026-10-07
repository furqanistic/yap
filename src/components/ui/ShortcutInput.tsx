import { IconKeyboard } from "@devigner-ui/icons/Keyboard";
import { useState, type KeyboardEvent } from "react";
import { getPlatform } from "@/lib/platform";
import { LiquidGlassSurface } from "@/components/liquid-glass";
import "./Field.css";

const MODIFIER_KEYS = new Set(["Control", "Alt", "Shift", "Meta"]);

const MODIFIER_LABELS =
  getPlatform() === "macos"
    ? { ctrl: "⌃", alt: "⌥", shift: "⇧", meta: "⌘" }
    : { ctrl: "Ctrl", alt: "Alt", shift: "Shift", meta: "Win" };

function modifiersOf(event: KeyboardEvent): string[] {
  const keys: string[] = [];
  if (event.ctrlKey) keys.push(MODIFIER_LABELS.ctrl);
  if (event.altKey) keys.push(MODIFIER_LABELS.alt);
  if (event.shiftKey) keys.push(MODIFIER_LABELS.shift);
  if (event.metaKey) keys.push(MODIFIER_LABELS.meta);
  return keys;
}

/** Readable name for the non-modifier key, based on its physical position. */
function keyLabel(event: KeyboardEvent): string {
  const { code, key } = event;
  if (code.startsWith("Key")) return code.slice(3);
  if (code.startsWith("Digit")) return code.slice(5);
  if (code === "Space") return "Space";
  return key.length === 1 ? key.toUpperCase() : key;
}

interface ShortcutInputProps {
  value: string[];
  onChange: (keys: string[]) => void;
  "aria-labelledby"?: string;
  "aria-describedby"?: string;
}

/**
 * Click, then press a key combination to record it. Holding only modifiers and
 * releasing them records a modifier-only shortcut. Escape cancels.
 */
export function ShortcutInput({ value, onChange, ...aria }: ShortcutInputProps) {
  const [recording, setRecording] = useState(false);
  const [pending, setPending] = useState<string[]>([]);

  const stopRecording = () => {
    setRecording(false);
    setPending([]);
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!recording) return;
    event.preventDefault();

    if (event.key === "Escape") {
      stopRecording();
      return;
    }

    const modifiers = modifiersOf(event);
    if (MODIFIER_KEYS.has(event.key)) {
      setPending(modifiers);
      return;
    }

    onChange([...modifiers, keyLabel(event)]);
    stopRecording();
  };

  const handleKeyUp = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!recording || pending.length === 0) return;
    event.preventDefault();
    onChange(pending);
    stopRecording();
  };

  const keys = recording ? pending : value;

  return (
    <LiquidGlassSurface
      as="button"
      type="button"
      preset="dense"
      className={`field field--shortcut${recording ? " field--recording" : ""}`}
      onClick={() => setRecording(true)}
      onKeyDown={handleKeyDown}
      onKeyUp={handleKeyUp}
      onBlur={stopRecording}
      {...aria}
      >
      <span className="field__keys" aria-live="polite">
        {keys.length > 0
          ? keys.map((key) => (
              <kbd key={key} className="field__key">
                {key}
              </kbd>
            ))
          : recording && <span className="field__placeholder">Press keys…</span>}
      </span>
      <IconKeyboard className="field__icon" strokeWidth={1.75} />
    </LiquidGlassSurface>
  );
}
