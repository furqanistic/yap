//! Microphone listing and the "Test microphone" level meter.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

use crate::audio::capture::{EndReason, Recorder, RecorderOptions};
use crate::audio::device::{self, InputDevice};
use crate::audio::AudioError;

pub const LEVEL_EVENT: &str = "audio://level";
pub const TEST_ENDED_EVENT: &str = "audio://test-ended";

/// A microphone test never needs to run longer than this.
const TEST_LIMIT: Duration = Duration::from_secs(5 * 60);

/// The running microphone test, if any.
#[derive(Default)]
pub struct MicTest(Mutex<Option<Recorder>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TestEnded {
    reason: EndReason,
    message: &'static str,
}

#[tauri::command]
pub async fn audio_list_devices() -> Result<Vec<InputDevice>, AudioError> {
    device::list()
}

/// Opens the microphone and emits `audio://level` about 30 times a second.
/// The audio itself is discarded.
#[tauri::command]
pub async fn audio_test_start(
    app: AppHandle,
    test: State<'_, MicTest>,
    device_id: String,
) -> Result<(), AudioError> {
    stop_test(&test);

    let level_app = app.clone();
    let end_app = app.clone();
    let recorder = Recorder::start(RecorderOptions {
        device_id,
        max_duration: TEST_LIMIT,
        keep_audio: false,
        on_level: Box::new(move |level| {
            let _ = level_app.emit(LEVEL_EVENT, level);
        }),
        on_end: Box::new(move |reason| {
            let message = match reason {
                EndReason::DeviceLost => "The microphone was disconnected.",
                EndReason::TimeLimit => "The test stopped after 5 minutes.",
            };
            let _ = end_app.emit(TEST_ENDED_EVENT, TestEnded { reason, message });
        }),
    })?;
    *test.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(recorder);
    Ok(())
}

#[tauri::command]
pub async fn audio_test_stop(test: State<'_, MicTest>) -> Result<(), AudioError> {
    stop_test(&test);
    Ok(())
}

fn stop_test(test: &MicTest) {
    let recorder = test.0.lock().unwrap_or_else(|e| e.into_inner()).take();
    if let Some(recorder) = recorder {
        recorder.stop();
    }
}

/// Opens the OS page where microphone access is granted.
#[tauri::command]
pub fn audio_open_privacy_settings(app: AppHandle) -> Result<(), String> {
    let url = if cfg!(target_os = "windows") {
        "ms-settings:privacy-microphone"
    } else if cfg!(target_os = "macos") {
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
    } else {
        return Err("Open your system's sound settings to allow microphone access.".into());
    };
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Debug builds only: records `seconds` of audio to a 16 kHz mono WAV file.
/// A scratch tool for testing transcription; not used by the UI.
#[tauri::command]
pub async fn audio_debug_record_wav(
    device_id: String,
    seconds: u64,
    path: String,
) -> Result<String, String> {
    if !cfg!(debug_assertions) {
        return Err("Only available in debug builds.".into());
    }
    let recorder = Recorder::start(RecorderOptions {
        device_id,
        max_duration: Duration::from_secs(seconds + 1),
        keep_audio: true,
        on_level: Box::new(|_| {}),
        on_end: Box::new(|_| {}),
    })
    .map_err(|e| e.to_string())?;
    std::thread::sleep(Duration::from_secs(seconds));
    let samples = recorder.stop();

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: crate::audio::TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
        writer.write_sample(value).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;
    Ok(path)
}
