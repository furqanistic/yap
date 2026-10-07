import type { SelectOption } from "@/components/ui";
import { getPlatform, getWindowAppearance, type WindowAppearance } from "@/lib/platform";

export type LanguageMode = "auto" | "preferred" | "translate";
export type TranscriptionModel = "whisper-tiny" | "whisper-base" | "whisper-small" | "whisper-large-turbo";

export interface GeneralSettingsValues {
  shortcut: string[];
  /** Device id, or "default" for the system microphone. */
  microphone: string;
  showRecordingIndicator: boolean;
  autoPaste: boolean;
  languageMode: LanguageMode;
  model: TranscriptionModel;
  launchAtLogin: boolean;
  startMinimized: boolean;
  windowAppearance: WindowAppearance;
}

export const DEFAULT_GENERAL_SETTINGS: GeneralSettingsValues = {
  // Ctrl+Space switches input sources on macOS, so default to Option+Space there.
  shortcut: getPlatform() === "macos" ? ["⌥", "Space"] : ["Ctrl", "Space"],
  microphone: "default",
  showRecordingIndicator: true,
  autoPaste: true,
  languageMode: "auto",
  model: "whisper-small",
  launchAtLogin: true,
  startMinimized: false,
  windowAppearance: getWindowAppearance(),
};

export const LANGUAGE_MODE_OPTIONS: readonly SelectOption<LanguageMode>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "preferred", label: "Use my preferred language" },
  { value: "translate", label: "Translate to English" },
];

export const WINDOW_APPEARANCE_OPTIONS: readonly SelectOption<WindowAppearance>[] = [
  { value: "glass", label: "Glass (blurred desktop behind Yap)" },
  { value: "solid", label: "Solid (best readability)" },
];

export const MODEL_OPTIONS: readonly SelectOption<TranscriptionModel>[] = [
  { value: "whisper-tiny", label: "Whisper Tiny (fastest)" },
  { value: "whisper-base", label: "Whisper Base (fast)" },
  { value: "whisper-small", label: "Whisper Small (balanced)" },
  { value: "whisper-large-turbo", label: "Whisper Large Turbo (accurate)" },
];

export const MODEL_HINTS: Record<TranscriptionModel, string> = {
  "whisper-tiny": "Near-instant results. Best for short, clear notes.",
  "whisper-base": "Quick and light. Good on older machines.",
  "whisper-small": "A good balance of speed and accuracy for everyday use.",
  "whisper-large-turbo": "Highest accuracy. Needs a fast machine.",
};
