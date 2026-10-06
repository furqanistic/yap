import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Button, Select, ShortcutInput, Switch, type SelectOption } from "@/components/ui";
import { ActiveModelSelect } from "@/features/model";
import { useAudioDevices } from "@/hooks/useAudioDevices";
import { useNavigation } from "@/hooks/useNavigation";
import { useSettings } from "@/hooks/useSettings";
import type { Settings } from "@/types/settings";
import { MicrophoneTest } from "./MicrophoneTest";
import { deviceLabel, LANGUAGE_MODE_OPTIONS } from "./options";
import "./GeneralSettings.css";

export function GeneralSettings() {
  const { settings, update: save } = useSettings();
  const { devices, error: devicesError } = useAudioDevices();
  const navigate = useNavigation();

  // Wait for the saved values so controls never flash the defaults.
  if (!settings) return null;

  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => save({ [key]: value });

  const microphoneOptions: SelectOption<string>[] = [
    { value: "default", label: "System default" },
    ...devices.map((device) => ({ value: device.id, label: deviceLabel(device.name) })),
  ];
  const microphoneMissing =
    settings.microphone !== "default" && !devices.some((device) => device.id === settings.microphone);
  const microphoneHint = devicesError
    ? devicesError.message
    : microphoneMissing
      ? "Not connected. Yap will use the system default."
      : undefined;

  return (
    <Page title="General" description="Configure how Yap looks and behaves on your system.">
      <SettingsGroup title="Recording">
        <SettingRow title="Push-to-talk shortcut" description="Hold this shortcut to start recording.">
          {(a11y) => (
            <ShortcutInput
              value={settings.holdShortcut}
              onChange={(keys) => update("holdShortcut", keys)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="Microphone"
          description="Select the microphone to use for recording."
          hint={microphoneHint}
        >
          {(a11y) => (
            <Select
              value={settings.microphone}
              options={microphoneOptions}
              placeholder="Not connected"
              onChange={(value) => update("microphone", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow title="Test microphone" description="Speak and watch the meter move.">
          {(a11y) => <MicrophoneTest deviceId={settings.microphone} {...a11y} />}
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

        <SettingRow title="Model" description="Select the transcription model to use.">
          {(a11y) => (
            <>
              <ActiveModelSelect {...a11y} />
              <Button size="small" variant="quiet" onClick={() => navigate("model")}>
                Manage models
              </Button>
            </>
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
