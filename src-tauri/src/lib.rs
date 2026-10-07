mod audio;
mod commands;
mod models;
mod store;
mod stt;

use tauri::{Emitter, Listener, Manager};

use models::ModelManager;
use store::settings::{SettingsFile, SettingsStore};
use stt::manager::SttManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                // whisper.cpp is chatty; keep its warnings and errors only.
                .level_for("whisper_rs", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            app.manage(SettingsStore::load(SettingsFile::new(config_dir)));

            let models_dir = app.path().app_data_dir()?.join("models");
            app.manage(ModelManager::new(models_dir));
            app.manage(commands::audio::MicTest::default());

            whisper_rs::install_logging_hooks();
            let status_app = app.handle().clone();
            app.manage(SttManager::new(
                stt::default_threads(),
                Box::new(move |status| {
                    let _ = status_app.emit(commands::stt::STATUS_EVENT, status);
                }),
            ));

            // Load the chosen model now, and again whenever it changes or
            // finishes downloading, so the first dictation doesn't wait.
            let handle = app.handle().clone();
            commands::stt::sync_model(&handle);
            for event in [commands::settings::SETTINGS_CHANGED, models::STATE_EVENT] {
                let handle = handle.clone();
                app.listen(event, move |_| commands::stt::sync_model(&handle));
            }
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
            commands::stt::stt_status,
            commands::stt::stt_transcribe_wav,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
