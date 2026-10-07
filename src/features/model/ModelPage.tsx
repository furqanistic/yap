import { IconFolderOpen } from "@devigner-ui/icons/FolderOpen";
import { IconStarsMinimalistic } from "@devigner-ui/icons/StarsMinimalistic";
import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Button, EmptyState, Select } from "@/components/ui";
import { useModels } from "@/hooks/useModels";
import { useSettings } from "@/hooks/useSettings";
import { formatBytes } from "@/lib/format";
import { isTauri } from "@/lib/tauri";
import { ActiveModelSelect } from "./ActiveModelSelect";
import { ModelRow } from "./ModelRow";
import { COMPUTE_DEVICE_OPTIONS } from "./options";
import "./ModelPage.css";

export function ModelPage() {
  const { models, loaded, dir, usedBytes, openFolder } = useModels();
  const { settings, update } = useSettings();

  if (!isTauri()) {
    return (
      <Page title="Model" description="Download and choose the speech model Yap uses.">
        <EmptyState
          icon={IconStarsMinimalistic}
          title="Models live in the desktop app"
          description="Open the Yap desktop app to download and manage models."
        />
      </Page>
    );
  }

  if (!loaded || !settings) return null;

  return (
    <Page title="Model" description="Download and choose the speech model Yap uses.">
      <SettingsGroup title="Active model">
        <SettingRow title="Model" description="The model used to turn your speech into text.">
          {(a11y) => <ActiveModelSelect {...a11y} />}
        </SettingRow>
        <SettingRow
          title="Compute device"
          description="Where transcription runs on this computer."
          hint="GPU support is coming in a later version."
        >
          {(a11y) => (
            <Select
              value={settings.computeDevice}
              options={COMPUTE_DEVICE_OPTIONS}
              onChange={(computeDevice) => update({ computeDevice })}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Available models">
        {models.map((model) => (
          <ModelRow key={model.id} model={model} />
        ))}
      </SettingsGroup>

      <SettingsGroup title="Storage">
        <SettingRow title="Models folder" description={dir}>
          {() => (
            <Button size="small" icon={IconFolderOpen} onClick={openFolder}>
              Open folder
            </Button>
          )}
        </SettingRow>
        <SettingRow title="Space used" description="Downloaded models and unfinished downloads.">
          {() => <span className="setting-row__value">{formatBytes(usedBytes)}</span>}
        </SettingRow>
      </SettingsGroup>
    </Page>
  );
}
