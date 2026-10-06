/**
 * Mirrors the Rust `Settings` struct in `src-tauri/src/store/settings.rs`.
 * Rust owns the defaults; add new fields there first, then here.
 */

export type LanguageMode = "auto" | "preferred" | "translate";

/** Ids from the Rust model catalog (`src-tauri/src/models/catalog.rs`). */
export type TranscriptionModel =
  | "whisper-tiny"
  | "whisper-tiny-en"
  | "whisper-base"
  | "whisper-base-en"
  | "whisper-small"
  | "whisper-small-en"
  | "whisper-large-turbo";

export type ComputeDevice = "auto" | "cpu" | "gpu";

export interface Settings {
  version: 1;
  /** Key labels as recorded by `ShortcutInput`, e.g. `["Ctrl", "Win"]`. */
  holdShortcut: string[];
  /** Device id, or "default" for the system microphone. */
  microphone: string;
  showRecordingIndicator: boolean;
  autoPaste: boolean;
  languageMode: LanguageMode;
  model: TranscriptionModel;
  computeDevice: ComputeDevice;
  launchAtLogin: boolean;
  startMinimized: boolean;
}

export type SettingsPatch = Partial<Omit<Settings, "version">>;
