import { useState } from "react";
import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Select, ShortcutInput, Switch, type SelectOption } from "@/components/ui";
import { useAudioInputDevices } from "@/hooks/useAudioInputDevices";
import { getPlatform, setWindowAppearance } from "@/lib/platform";
import {
  DEFAULT_GENERAL_SETTINGS,
  LANGUAGE_MODE_OPTIONS,
  MODEL_HINTS,
  MODEL_OPTIONS,
  WINDOW_APPEARANCE_OPTIONS,
  type GeneralSettingsValues,
} from "./options";

export function GeneralSettings() {
  // TODO: persist settings and apply them through the Rust backend.
  const [settings, setSettings] = useState<GeneralSettingsValues>(DEFAULT_GENERAL_SETTINGS);
  const devices = useAudioInputDevices();

  const update = <K extends keyof GeneralSettingsValues>(key: K, value: GeneralSettingsValues[K]) =>
    setSettings((current) => ({ ...current, [key]: value }));

  const microphoneOptions: SelectOption<string>[] = [
    { value: "default", label: "System default" },
    ...devices.map((device) => ({ value: device.id, label: device.label })),
  ];

  return (
    <Page title="General" description="Configure how Yap looks and behaves on your system.">
      <SettingsGroup title="Appearance">
        <SettingRow
          title="Window appearance"
          description={
            getPlatform() === "linux"
              ? "Glass is unavailable on Linux, so Yap stays solid for readability."
              : "Glass shows a blurred hint of the desktop behind Yap. Solid maximizes readability."
          }
        >
          {(a11y) => (
            <Select
              value={settings.windowAppearance}
              options={WINDOW_APPEARANCE_OPTIONS}
              onChange={(value) => {
                update("windowAppearance", value);
                setWindowAppearance(value);
              }}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Recording">
        <SettingRow title="Push-to-talk shortcut" description="Hold this shortcut to start recording.">
          {(a11y) => (
            <ShortcutInput
              value={settings.shortcut}
              onChange={(keys) => update("shortcut", keys)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow title="Microphone" description="Select the microphone to use for recording.">
          {(a11y) => (
            <Select
              value={settings.microphone}
              options={microphoneOptions}
              onChange={(value) => update("microphone", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow title="Show recording indicator" description="Display a small indicator while recording.">
          {(a11y) => (
            <Switch
              checked={settings.showRecordingIndicator}
              onChange={(value) => update("showRecordingIndicator", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Transcription">
        <SettingRow
          title="Auto-paste transcribed text"
          description="Automatically paste the transcription at your cursor."
        >
          {(a11y) => (
            <Switch
              checked={settings.autoPaste}
              onChange={(value) => update("autoPaste", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow title="Language mode" description="Choose how to handle transcription language.">
          {(a11y) => (
            <Select
              value={settings.languageMode}
              options={LANGUAGE_MODE_OPTIONS}
              onChange={(value) => update("languageMode", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="Model"
          description="Select the transcription model to use."
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
      </SettingsGroup>

      <SettingsGroup title="Startup">
        <SettingRow title="Launch at login" description="Automatically start Yap when you log in.">
          {(a11y) => (
            <Switch
              checked={settings.launchAtLogin}
              onChange={(value) => update("launchAtLogin", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow title="Start minimized" description="Open Yap in the background when it starts.">
          {(a11y) => (
            <Switch
              checked={settings.startMinimized}
              onChange={(value) => update("startMinimized", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>
    </Page>
  );
}
