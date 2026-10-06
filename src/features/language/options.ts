import type { SelectOption } from "@/components/ui";

export type LanguageCode =
  | "auto"
  | "en"
  | "es"
  | "fr"
  | "de"
  | "it"
  | "pt"
  | "nl"
  | "zh"
  | "ja"
  | "ko"
  | "ar"
  | "hi";

export interface LanguageSettingsValues {
  primaryLanguage: LanguageCode;
  autoDetect: boolean;
  translateToEnglish: boolean;
  smartPunctuation: boolean;
  formatNumbers: boolean;
}

export const DEFAULT_LANGUAGE_SETTINGS: LanguageSettingsValues = {
  primaryLanguage: "en",
  autoDetect: true,
  translateToEnglish: false,
  smartPunctuation: true,
  formatNumbers: true,
};

export const LANGUAGE_OPTIONS: readonly SelectOption<LanguageCode>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "en", label: "English" },
  { value: "es", label: "Spanish (Español)" },
  { value: "fr", label: "French (Français)" },
  { value: "de", label: "German (Deutsch)" },
  { value: "it", label: "Italian (Italiano)" },
  { value: "pt", label: "Portuguese (Português)" },
  { value: "nl", label: "Dutch (Nederlands)" },
  { value: "zh", label: "Chinese (中文)" },
  { value: "ja", label: "Japanese (日本語)" },
  { value: "ko", label: "Korean (한국어)" },
  { value: "ar", label: "Arabic (العربية)" },
  { value: "hi", label: "Hindi (हिन्दी)" },
];
