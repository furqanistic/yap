import type { SelectOption } from "@/components/ui";
import type { LanguageMode } from "@/types/settings";

// Default values live in Rust (`src-tauri/src/store/settings.rs`).
// Model options come from the model catalog (see `features/model`).

export const LANGUAGE_MODE_OPTIONS: readonly SelectOption<LanguageMode>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "preferred", label: "Use my preferred language" },
  { value: "translate", label: "Translate to English" },
];

/**
 * Shorter device names for the 240px field. Windows wraps most names as
 * "Microphone (Device name)"; the device name alone is clearer.
 * deviceLabel("Microphone (HD Webcam C960)") → "HD Webcam C960"
 */
export function deviceLabel(name: string): string {
  const match = /^(?:Microphone|Microphone Array|Headset Microphone) \((.+)\)$/.exec(name);
  return match ? match[1] : name;
}
