mod audio;
mod commands;
mod models;
mod store;

use tauri::Manager;

use models::ModelManager;
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

            let models_dir = app.path().app_data_dir()?.join("models");
            app.manage(ModelManager::new(models_dir));
            app.manage(commands::audio::MicTest::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::settings::settings_reset,
            commands::models::models_list,
            commands::models::models_dir,
            commands::models::models_disk_space,
            commands::models::models_download,
            commands::models::models_cancel,
            commands::models::models_delete,
            commands::models::models_open_folder,
            commands::audio::audio_list_devices,
            commands::audio::audio_test_start,
            commands::audio::audio_test_stop,
            commands::audio::audio_open_privacy_settings,
            commands::audio::audio_debug_record_wav,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
