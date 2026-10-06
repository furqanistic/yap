import type { SelectOption } from "@/components/ui";
import type { ModelId, ModelLanguages } from "@/types/models";
import type { ComputeDevice } from "@/types/settings";

/** Suggested first download, used by "Download a model". */
export const RECOMMENDED_MODEL: ModelId = "whisper-small";

export const LANGUAGE_TAGS: Record<ModelLanguages, string> = {
  englishOnly: "English only",
  multilingual: "Multilingual",
};

// GPU stays disabled until GPU acceleration lands.
export const COMPUTE_DEVICE_OPTIONS: readonly SelectOption<ComputeDevice>[] = [
  { value: "auto", label: "Auto (recommended)" },
  { value: "cpu", label: "CPU only" },
  { value: "gpu", label: "GPU (coming soon)", disabled: true },
];
