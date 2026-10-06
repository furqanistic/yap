import type { SelectOption } from "@/components/ui";

export type ModelId =
  | "whisper-tiny"
  | "whisper-base"
  | "whisper-small"
  | "whisper-medium"
  | "whisper-large-turbo";

export type ComputeDevice = "auto" | "cpu" | "gpu";

export interface ModelSettingsValues {
  model: ModelId;
  computeDevice: ComputeDevice;
  quantization8Bit: boolean;
  keepInMemory: boolean;
}

export const DEFAULT_MODEL_SETTINGS: ModelSettingsValues = {
  model: "whisper-small",
  computeDevice: "auto",
  quantization8Bit: true,
  keepInMemory: true,
};

export const MODEL_OPTIONS: readonly SelectOption<ModelId>[] = [
  { value: "whisper-tiny", label: "Whisper Tiny (~39 MB)" },
  { value: "whisper-base", label: "Whisper Base (~74 MB)" },
  { value: "whisper-small", label: "Whisper Small (~244 MB, recommended)" },
  { value: "whisper-medium", label: "Whisper Medium (~769 MB)" },
  { value: "whisper-large-turbo", label: "Whisper Large Turbo (~1.5 GB)" },
];

export const COMPUTE_DEVICE_OPTIONS: readonly SelectOption<ComputeDevice>[] = [
  { value: "auto", label: "Auto-detect (recommended)" },
  { value: "gpu", label: "GPU acceleration" },
  { value: "cpu", label: "CPU only" },
];

export const MODEL_HINTS: Record<ModelId, string> = {
  "whisper-tiny": "Near-instant transcription. Best for short, clear notes on low-power devices.",
  "whisper-base": "Quick and lightweight. Good balance for older laptops.",
  "whisper-small": "High accuracy and quick response time. Recommended for daily dictation.",
  "whisper-medium": "High fidelity for complex terminology and accented speech.",
  "whisper-large-turbo": "Maximum accuracy across multilingual speech. Requires 4+ GB VRAM/RAM.",
};
