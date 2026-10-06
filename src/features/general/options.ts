import type { SelectOption } from "@/components/ui";
import type { LanguageMode, TranscriptionModel } from "@/types/settings";

// Default values live in Rust (`src-tauri/src/store/settings.rs`).

export const LANGUAGE_MODE_OPTIONS: readonly SelectOption<LanguageMode>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "preferred", label: "Use my preferred language" },
  { value: "translate", label: "Translate to English" },
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
