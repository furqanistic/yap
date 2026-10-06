import { useState } from "react";
import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Select, Switch } from "@/components/ui";
import {
  COMPUTE_DEVICE_OPTIONS,
  DEFAULT_MODEL_SETTINGS,
  MODEL_HINTS,
  MODEL_OPTIONS,
  type ModelSettingsValues,
} from "./options";

export function ModelSettings() {
  const [settings, setSettings] = useState<ModelSettingsValues>(DEFAULT_MODEL_SETTINGS);

  const update = <K extends keyof ModelSettingsValues>(key: K, value: ModelSettingsValues[K]) =>
    setSettings((current) => ({ ...current, [key]: value }));

  return (
    <Page
      title="Model"
      description="Choose the AI model and hardware acceleration used for transcription."
    >
      <SettingsGroup title="Model selection">
        <SettingRow
          title="Transcription model"
          description="Select the speech-to-text model weight."
          hint={MODEL_HINTS[settings.model]}
        >
          {(a11y) => (
            <Select
              value={settings.model}
              options={MODEL_OPTIONS}
              onChange={(value) => update("model", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="Keep model in memory"
          description="Retain the model in RAM after transcription for instant follow-up dictations."
        >
          {(a11y) => (
            <Switch
              checked={settings.keepInMemory}
              onChange={(value) => update("keepInMemory", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Hardware acceleration">
        <SettingRow
          title="Compute device"
          description="Hardware backend used to run inference."
        >
          {(a11y) => (
            <Select
              value={settings.computeDevice}
              options={COMPUTE_DEVICE_OPTIONS}
              onChange={(value) => update("computeDevice", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="8-bit quantization"
          description="Reduce memory usage with negligible impact on transcription accuracy."
        >
          {(a11y) => (
            <Switch
              checked={settings.quantization8Bit}
              onChange={(value) => update("quantization8Bit", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>
    </Page>
  );
}
