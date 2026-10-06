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
  { value: "whisper-tiny-en", label: "Whisper Tiny English" },
  { value: "whisper-base", label: "Whisper Base (fast)" },
  { value: "whisper-base-en", label: "Whisper Base English" },
  { value: "whisper-small", label: "Whisper Small (balanced)" },
  { value: "whisper-small-en", label: "Whisper Small English" },
  { value: "whisper-large-turbo", label: "Whisper Large Turbo (accurate)" },
];

export const MODEL_HINTS: Record<TranscriptionModel, string> = {
  "whisper-tiny": "Near-instant results. Best for short, clear notes.",
  "whisper-tiny-en": "Near-instant results, a little more accurate for English.",
  "whisper-base": "Quick and light. Good on older machines.",
  "whisper-base-en": "Quick and light, a little more accurate for English.",
  "whisper-small": "A good balance of speed and accuracy for everyday use.",
  "whisper-small-en": "Balanced speed and accuracy, tuned for English.",
  "whisper-large-turbo": "Highest accuracy. Needs a fast machine.",
};
