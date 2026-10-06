//! Tauri commands exposed to the frontend via `invoke`.
//! Add one submodule per domain (e.g. `audio`, `transcription`) and register
//! its commands in `lib.rs` with `tauri::generate_handler!`.

pub mod models;
pub mod settings;
