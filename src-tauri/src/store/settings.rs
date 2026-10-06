//! User settings: defaults, loading, atomic saving and JSON patching.
//!
//! Rust owns the defaults. The frontend mirrors the shape in
//! `src/types/settings.ts`. To add a setting, add the field and its default
//! here, then mirror it in TypeScript. `#[serde(default)]` keeps older files
//! loading when new fields appear.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const SETTINGS_VERSION: u32 = 1;
const FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageMode {
    Auto,
    Preferred,
    Translate,
}

/// Where transcription runs. GPU options arrive in a later version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComputeDevice {
    Auto,
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    /// Key labels as recorded by the UI, e.g. `["Ctrl", "Win"]`.
    pub hold_shortcut: Vec<String>,
    /// Input device id, or `"default"` for the system microphone.
    pub microphone: String,
    pub show_recording_indicator: bool,
    pub auto_paste: bool,
    pub language_mode: LanguageMode,
    pub model: String,
    pub compute_device: ComputeDevice,
    pub launch_at_login: bool,
    pub start_minimized: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            hold_shortcut: default_shortcut(),
            microphone: "default".into(),
            show_recording_indicator: true,
            auto_paste: true,
            language_mode: LanguageMode::Auto,
            model: "whisper-small".into(),
            compute_device: ComputeDevice::Auto,
            launch_at_login: true,
            start_minimized: false,
        }
    }
}

/// Ctrl+Space switches input sources on macOS, so default to Option+Space there.
fn default_shortcut() -> Vec<String> {
    if cfg!(target_os = "macos") {
        vec!["\u{2325}".into(), "Space".into()]
    } else {
        vec!["Ctrl".into(), "Space".into()]
    }
}

#[derive(Debug)]
pub enum SettingsError {
    Io(io::Error),
    Invalid(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "could not save settings: {err}"),
            Self::Invalid(message) => write!(f, "invalid settings: {message}"),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl Settings {
    /// Returns a copy with `patch` (a partial JSON object) merged in.
    /// Fails if the result doesn't match the settings shape.
    pub fn patched(&self, patch: &Value) -> Result<Self, SettingsError> {
        let Value::Object(patch) = patch else {
            return Err(SettingsError::Invalid("patch must be an object".into()));
        };
        let mut merged =
            serde_json::to_value(self).map_err(|e| SettingsError::Invalid(e.to_string()))?;
        if let Value::Object(fields) = &mut merged {
            for (key, value) in patch {
                if key != "version" {
                    fields.insert(key.clone(), value.clone());
                }
            }
        }
        serde_json::from_value(merged).map_err(|e| SettingsError::Invalid(e.to_string()))
    }

    /// Upgrades settings saved by an older version of Yap.
    fn migrate(mut self) -> Self {
        // No migrations yet. Add `if self.version < N { ... }` blocks here.
        self.version = SETTINGS_VERSION;
        self
    }
}

/// Reads and writes `settings.json` in a directory.
#[derive(Debug, Clone)]
pub struct SettingsFile {
    dir: PathBuf,
}

impl SettingsFile {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn path(&self) -> PathBuf {
        self.dir.join(FILE_NAME)
    }

    /// Loads settings, falling back to defaults when the file is missing.
    /// A corrupt file is kept as `settings.json.bak` and replaced by defaults.
    pub fn load(&self) -> Settings {
        let path = self.path();
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Settings::default(),
            Err(err) => {
                log::warn!("could not read {}: {err}", path.display());
                return Settings::default();
            }
        };

        match serde_json::from_str::<Settings>(&text) {
            Ok(settings) => settings.migrate(),
            Err(err) => {
                log::warn!("settings file is corrupt ({err}); using defaults");
                let _ = fs::rename(&path, self.dir.join(format!("{FILE_NAME}.bak")));
                Settings::default()
            }
        }
    }

    /// Saves atomically: writes a temp file, then renames it over the old one.
    pub fn save(&self, settings: &Settings) -> Result<(), SettingsError> {
        fs::create_dir_all(&self.dir)?;
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| SettingsError::Invalid(e.to_string()))?;
        write_atomic(&self.path(), json.as_bytes())?;
        Ok(())
    }
}

/// The live settings, held in Tauri managed state and saved on every change.
pub struct SettingsStore {
    file: SettingsFile,
    current: Mutex<Settings>,
}

impl SettingsStore {
    pub fn load(file: SettingsFile) -> Self {
        let current = Mutex::new(file.load());
        Self { file, current }
    }

    pub fn get(&self) -> Settings {
        self.lock().clone()
    }

    /// Merges `patch`, saves, and returns the new settings.
    pub fn update(&self, patch: &Value) -> Result<Settings, SettingsError> {
        let mut current = self.lock();
        let next = current.patched(patch)?;
        self.file.save(&next)?;
        *current = next.clone();
        Ok(next)
    }

    pub fn reset(&self) -> Result<Settings, SettingsError> {
        let mut current = self.lock();
        let next = Settings::default();
        self.file.save(&next)?;
        *current = next.clone();
        Ok(next)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Settings> {
        // A panic while holding the lock can't leave settings half-written,
        // so recover the value instead of poisoning every later call.
        self.current.lock().unwrap_or_else(|e| e.into_inner())
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("yap-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn defaults_are_sensible() {
        let settings = Settings::default();
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert_eq!(settings.microphone, "default");
        assert_eq!(settings.model, "whisper-small");
        assert_eq!(settings.language_mode, LanguageMode::Auto);
        assert_eq!(settings.hold_shortcut.len(), 2);
    }

    #[test]
    fn serializes_as_camel_case() {
        let value = serde_json::to_value(Settings::default()).unwrap();
        assert!(value.get("holdShortcut").is_some());
        assert!(value.get("showRecordingIndicator").is_some());
        assert_eq!(value["languageMode"], "auto");
    }

    #[test]
    fn patch_merges_only_given_fields() {
        let settings = Settings::default();
        let patched = settings
            .patched(&json!({ "autoPaste": false, "holdShortcut": ["Ctrl", "Win"] }))
            .unwrap();
        assert!(!patched.auto_paste);
        assert_eq!(patched.hold_shortcut, vec!["Ctrl", "Win"]);
        assert_eq!(patched.model, settings.model);
    }

    #[test]
    fn patch_rejects_wrong_types_and_ignores_version() {
        let settings = Settings::default();
        assert!(settings.patched(&json!({ "autoPaste": "yes" })).is_err());
        assert!(settings.patched(&json!(["not", "an", "object"])).is_err());
        assert!(settings
            .patched(&json!({ "languageMode": "klingon" }))
            .is_err());
        let patched = settings.patched(&json!({ "version": 99 })).unwrap();
        assert_eq!(patched.version, SETTINGS_VERSION);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let file = SettingsFile::new(temp_dir("missing"));
        assert_eq!(file.load(), Settings::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let file = SettingsFile::new(temp_dir("roundtrip"));
        let settings = Settings {
            microphone: "USB Mic".into(),
            start_minimized: true,
            ..Settings::default()
        };
        file.save(&settings).unwrap();
        assert_eq!(file.load(), settings);
        assert!(!file.path().with_extension("json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_backed_up_and_defaults_used() {
        let dir = temp_dir("corrupt");
        let file = SettingsFile::new(&dir);
        fs::write(file.path(), "{ not json").unwrap();
        assert_eq!(file.load(), Settings::default());
        assert!(dir.join("settings.json.bak").exists());
    }

    #[test]
    fn unknown_and_missing_fields_are_tolerated() {
        let dir = temp_dir("unknown");
        let file = SettingsFile::new(&dir);
        fs::write(
            file.path(),
            r#"{ "version": 1, "autoPaste": false, "fromTheFuture": 42 }"#,
        )
        .unwrap();
        let settings = file.load();
        assert!(!settings.auto_paste);
        assert_eq!(settings.model, "whisper-small");
    }

    #[test]
    fn store_update_persists_and_rejects_bad_patches() {
        let dir = temp_dir("store");
        let store = SettingsStore::load(SettingsFile::new(&dir));
        store.update(&json!({ "launchAtLogin": false })).unwrap();
        assert!(store.update(&json!({ "launchAtLogin": 3 })).is_err());
        assert!(!store.get().launch_at_login);
        assert!(!SettingsFile::new(&dir).load().launch_at_login);
        store.reset().unwrap();
        assert_eq!(store.get(), Settings::default());
    }
}
