//! Debug tooling for text insertion. The dictation pipeline calls
//! `inject::insert` directly; the UI doesn't use this.

use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::inject::{self, InsertOptions, InsertResult};
use crate::store::settings::SettingsStore;

/// Debug builds only: waits `delay_ms` (time to focus another app), then
/// inserts `text` there using the current settings.
#[tauri::command]
pub async fn inject_text(
    app: AppHandle,
    text: String,
    delay_ms: u64,
) -> Result<InsertResult, String> {
    if !cfg!(debug_assertions) {
        return Err("Only available in debug builds.".into());
    }
    let options = InsertOptions::from_settings(&app.state::<SettingsStore>().get());
    tauri::async_runtime::spawn_blocking(move || {
        std::thread::sleep(Duration::from_millis(delay_ms));
        inject::insert(&text, options)
    })
    .await
    .map_err(|e| e.to_string())
}
