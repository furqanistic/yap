import { useState } from "react";
import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Select, Switch } from "@/components/ui";
import {
  DEFAULT_LANGUAGE_SETTINGS,
  LANGUAGE_OPTIONS,
  type LanguageSettingsValues,
} from "./options";

export function LanguageSettings() {
  const [settings, setSettings] = useState<LanguageSettingsValues>(DEFAULT_LANGUAGE_SETTINGS);

  const update = <K extends keyof LanguageSettingsValues>(key: K, value: LanguageSettingsValues[K]) =>
    setSettings((current) => ({ ...current, [key]: value }));

  return (
    <Page title="Language" description="Pick the languages you dictate in and configure text formatting.">
      <SettingsGroup title="Spoken language">
        <SettingRow
          title="Primary language"
          description="Default language used for recognition and vocabulary matching."
        >
          {(a11y) => (
            <Select
              value={settings.primaryLanguage}
              options={LANGUAGE_OPTIONS}
              onChange={(value) => update("primaryLanguage", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="Auto-detect language"
          description="Automatically recognize speech in other languages when dictated."
        >
          {(a11y) => (
            <Switch
              checked={settings.autoDetect}
              onChange={(value) => update("autoDetect", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Translation">
        <SettingRow
          title="Translate to English"
          description="Automatically translate speech from other languages into English."
        >
          {(a11y) => (
            <Switch
              checked={settings.translateToEnglish}
              onChange={(value) => update("translateToEnglish", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Formatting">
        <SettingRow
          title="Automatic punctuation"
          description="Insert commas, periods, and question marks based on speech cadence."
        >
          {(a11y) => (
            <Switch
              checked={settings.smartPunctuation}
              onChange={(value) => update("smartPunctuation", value)}
              {...a11y}
            />
          )}
        </SettingRow>

        <SettingRow
          title="Format numbers as digits"
          description="Write numbers as digits (e.g., 42) rather than spelled-out words."
        >
          {(a11y) => (
            <Switch
              checked={settings.formatNumbers}
              onChange={(value) => update("formatNumbers", value)}
              {...a11y}
            />
          )}
        </SettingRow>
      </SettingsGroup>
    </Page>
  );
}
