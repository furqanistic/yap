//! Reading and changing user settings.

use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

use crate::store::settings::{Settings, SettingsStore};

/// Emitted with the full settings after every change.
pub const SETTINGS_CHANGED: &str = "settings://changed";

#[tauri::command]
pub fn settings_get(store: State<'_, SettingsStore>) -> Settings {
    store.get()
}

/// Merges a partial settings object into the current settings and saves it.
#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    store: State<'_, SettingsStore>,
    patch: Value,
) -> Result<Settings, String> {
    let settings = store.update(&patch).map_err(|e| e.to_string())?;
    let _ = app.emit(SETTINGS_CHANGED, &settings);
    Ok(settings)
}

#[tauri::command]
pub fn settings_reset(app: AppHandle, store: State<'_, SettingsStore>) -> Result<Settings, String> {
    let settings = store.reset().map_err(|e| e.to_string())?;
    let _ = app.emit(SETTINGS_CHANGED, &settings);
    Ok(settings)
}
