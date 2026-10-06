//! Listing, downloading and deleting speech models.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::models::download::ModelError;
use crate::models::{ModelEntry, ModelManager};

#[tauri::command]
pub fn models_list(models: State<'_, ModelManager>) -> Vec<ModelEntry> {
    models.list()
}

#[tauri::command]
pub fn models_dir(models: State<'_, ModelManager>) -> String {
    models.dir().display().to_string()
}

/// Starts a download and returns right away. Watch `models://progress` and
/// `models://state` for the rest.
#[tauri::command]
pub fn models_download(
    app: AppHandle,
    models: State<'_, ModelManager>,
    id: String,
) -> Result<(), ModelError> {
    models.start_download(&app, &id)
}

#[tauri::command]
pub fn models_cancel(models: State<'_, ModelManager>, id: String) {
    models.cancel(&id);
}

#[tauri::command]
pub fn models_delete(
    app: AppHandle,
    models: State<'_, ModelManager>,
    id: String,
) -> Result<(), ModelError> {
    models.delete(&app, &id)
}

#[tauri::command]
pub fn models_open_folder(app: AppHandle, models: State<'_, ModelManager>) -> Result<(), String> {
    std::fs::create_dir_all(models.dir()).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(models.dir().display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}
