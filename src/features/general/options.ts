import type { SelectOption } from "@/components/ui";
import type { LanguageMode } from "@/types/settings";

// Default values live in Rust (`src-tauri/src/store/settings.rs`).
// Model options come from the model catalog (see `features/model`).

export const LANGUAGE_MODE_OPTIONS: readonly SelectOption<LanguageMode>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "preferred", label: "Use my preferred language" },
  { value: "translate", label: "Translate to English" },
];
