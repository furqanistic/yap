mod commands;
mod store;

use tauri::Manager;

use store::settings::{SettingsFile, SettingsStore};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(SettingsStore::load(SettingsFile::new(config_dir)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_reset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
