//! Transcription engine status, plus a debug-only WAV transcriber.

use tauri::{AppHandle, Manager, State};

use crate::audio::capture::{mix_to_mono, StreamResampler};
use crate::models::catalog::{self, ModelInfo};
use crate::models::ModelManager;
use crate::store::settings::SettingsStore;
use crate::stt::engine::Transcript;
use crate::stt::manager::{ModelFile, SttManager, SttStatus};
use crate::stt::SttError;

pub const STATUS_EVENT: &str = "stt://status";

#[tauri::command]
pub fn stt_status(stt: State<'_, SttManager>) -> SttStatus {
    stt.status()
}

/// The model chosen in settings, if it's downloaded.
pub fn active_model(app: &AppHandle) -> Result<(ModelFile, &'static ModelInfo), SttError> {
    let id = app.state::<SettingsStore>().get().model;
    let info = catalog::find(&id).ok_or(SttError::NoModel)?;
    let path = app
        .state::<ModelManager>()
        .ready_path(&id)
        .ok_or(SttError::NoModel)?;
    Ok((ModelFile { id, path }, info))
}

/// Keeps the loaded model in step with settings: loads the chosen model
/// when it's downloaded, unloads when it's gone. Called at startup and
/// whenever settings or downloads change.
pub fn sync_model(app: &AppHandle) {
    let stt = app.state::<SttManager>();
    match active_model(app) {
        Ok((file, _)) => {
            if stt.status().model.as_deref() != Some(file.id.as_str()) {
                stt.load(file);
            }
        }
        Err(_) => {
            if stt.status().model.is_some() {
                stt.unload();
            }
        }
    }
}

/// Debug builds only: transcribes a WAV file with the active model and
/// settings. A testing aid; the UI doesn't use it.
#[tauri::command]
pub async fn stt_transcribe_wav(app: AppHandle, path: String) -> Result<Transcript, String> {
    if !cfg!(debug_assertions) {
        return Err("Only available in debug builds.".into());
    }
    let samples = read_wav_16k_mono(&path)?;
    let (file, info) = active_model(&app).map_err(|e| e.to_string())?;
    let options = crate::stt::options_for(&app.state::<SettingsStore>().get(), info)
        .map_err(|e| e.to_string())?;
    app.state::<SttManager>()
        .transcribe(file, samples, options)
        .await
        .map_err(|e| e.to_string())
}

fn read_wav_16k_mono(path: &str) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let interleaved: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        hound::SampleFormat::Int => {
            let scale = (1_i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.unwrap_or(0) as f32 / scale)
                .collect()
        }
    };
    let mut mono = Vec::new();
    mix_to_mono(&interleaved, usize::from(spec.channels), &mut mono);
    let mut resampler = StreamResampler::new(spec.sample_rate);
    resampler.push(&mono);
    Ok(resampler.finish())
}
