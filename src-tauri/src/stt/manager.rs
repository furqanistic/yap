//! Owns the loaded model on a dedicated worker thread.
//!
//! Loading and transcribing take seconds, so they never run on the Tauri
//! main thread or an async runtime thread. Callers send jobs over a channel
//! and await the reply.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tokio::sync::oneshot;

use super::engine::{Engine, TranscribeOptions, Transcript};
use super::{filter, SttError};
use crate::audio::{capture, vad};

/// What the engine is doing, for `stt_status` and `stt://status`.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SttStatus {
    /// The model that's loaded and ready.
    pub model: Option<String>,
    /// The model being loaded right now.
    pub loading: Option<String>,
    pub error: Option<String>,
}

/// A downloaded model on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFile {
    pub id: String,
    pub path: PathBuf,
}

enum Job {
    Load(ModelFile),
    Unload,
    Transcribe {
        model: ModelFile,
        samples: Vec<f32>,
        options: TranscribeOptions,
        reply: oneshot::Sender<Result<Transcript, SttError>>,
    },
}

type StatusListener = Box<dyn Fn(&SttStatus) + Send>;

pub struct SttManager {
    jobs: Mutex<Sender<Job>>,
    status: Arc<Mutex<SttStatus>>,
}

impl SttManager {
    pub fn new(threads: usize, on_status: StatusListener) -> Self {
        let (tx, rx) = mpsc::channel();
        let status = Arc::new(Mutex::new(SttStatus::default()));
        let worker_status = status.clone();
        std::thread::Builder::new()
            .name("yap-stt".into())
            .spawn(move || worker(rx, threads, worker_status, on_status))
            .expect("can start the transcription thread");
        Self {
            jobs: Mutex::new(tx),
            status,
        }
    }

    pub fn status(&self) -> SttStatus {
        self.status
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Loads a model in the background, replacing the current one.
    pub fn load(&self, model: ModelFile) {
        self.send(Job::Load(model));
    }

    pub fn unload(&self) {
        self.send(Job::Unload);
    }

    /// Transcribes 16 kHz mono audio with `model`, loading it first if
    /// needed. Audio without speech returns an empty transcript and never
    /// reaches Whisper.
    pub async fn transcribe(
        &self,
        model: ModelFile,
        samples: Vec<f32>,
        options: TranscribeOptions,
    ) -> Result<Transcript, SttError> {
        let (reply, response) = oneshot::channel();
        self.send(Job::Transcribe {
            model,
            samples,
            options,
            reply,
        });
        response
            .await
            .unwrap_or_else(|_| Err(SttError::Transcribe("the engine stopped".into())))
    }

    fn send(&self, job: Job) {
        let _ = self
            .jobs
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .send(job);
    }
}

fn worker(
    jobs: Receiver<Job>,
    threads: usize,
    status: Arc<Mutex<SttStatus>>,
    on_status: StatusListener,
) {
    let mut loaded: Option<(ModelFile, Engine)> = None;

    let set_status = |update: &dyn Fn(&mut SttStatus)| {
        let snapshot = {
            let mut current = status.lock().unwrap_or_else(|e| e.into_inner());
            update(&mut current);
            current.clone()
        };
        on_status(&snapshot);
    };

    let ensure_loaded = |model: &ModelFile, loaded: &mut Option<(ModelFile, Engine)>| {
        if loaded.as_ref().is_some_and(|(file, _)| file == model) {
            return Ok(());
        }
        // Free the old model before loading the next one.
        *loaded = None;
        set_status(&|s| {
            s.model = None;
            s.loading = Some(model.id.clone());
            s.error = None;
        });
        match Engine::load(&model.path, threads) {
            Ok(engine) => {
                log::info!("loaded model {}", model.id);
                *loaded = Some((model.clone(), engine));
                set_status(&|s| {
                    s.model = Some(model.id.clone());
                    s.loading = None;
                });
                Ok(())
            }
            Err(error) => {
                log::warn!("couldn't load {}: {error}", model.id);
                let message = error.to_string();
                set_status(&|s| {
                    s.loading = None;
                    s.error = Some(message.clone());
                });
                Err(error)
            }
        }
    };

    for job in jobs {
        match job {
            Job::Load(model) => {
                let _ = ensure_loaded(&model, &mut loaded);
            }
            Job::Unload => {
                loaded = None;
                set_status(&|s| *s = SttStatus::default());
            }
            Job::Transcribe {
                model,
                samples,
                options,
                reply,
            } => {
                let result = ensure_loaded(&model, &mut loaded).and_then(|()| {
                    let (_, engine) = loaded.as_mut().expect("just loaded");
                    transcribe(engine, &samples, &options)
                });
                let _ = reply.send(result);
            }
        }
    }
}

fn transcribe(
    engine: &mut Engine,
    samples: &[f32],
    options: &TranscribeOptions,
) -> Result<Transcript, SttError> {
    if !vad::has_speech(samples) {
        log::info!("no speech detected; skipping Whisper");
        return Ok(Transcript::default());
    }
    let speech = vad::trim_silence(samples);
    let mut transcript = engine.transcribe(speech, options)?;
    if filter::is_hallucination(&transcript.text, capture::rms(speech)) {
        log::info!("dropped a likely hallucination: {:?}", transcript.text);
        transcript.text.clear();
        transcript.segments.clear();
    }
    Ok(transcript)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stt::engine::Language;

    /// `YAP_MODEL=... YAP_WAV=... cargo test -- --ignored gate_with_real_model --nocapture`
    #[test]
    #[ignore = "needs a downloaded model and a WAV file"]
    fn gate_with_real_model() {
        let model = std::env::var("YAP_MODEL").expect("set YAP_MODEL");
        let wav = std::env::var("YAP_WAV").expect("set YAP_WAV");
        let mut engine = Engine::load(std::path::Path::new(&model), 4).unwrap();
        let options = TranscribeOptions {
            language: Language::Auto,
            translate: false,
        };

        let silence = vec![0.0; 16_000 * 3];
        let mut state = 1u32;
        let hiss: Vec<f32> = (0..16_000 * 3)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state as f32 / u32::MAX as f32 - 0.5) * 0.02
            })
            .collect();
        for (name, audio) in [("silence", silence), ("hiss", hiss)] {
            let transcript = transcribe(&mut engine, &audio, &options).unwrap();
            println!(
                "{name}: {:?} ({} ms)",
                transcript.text, transcript.duration_ms
            );
            assert!(transcript.text.is_empty(), "{name}");
            assert_eq!(transcript.duration_ms, 0, "{name} must not reach Whisper");
        }

        let speech: Vec<f32> = hound::WavReader::open(wav)
            .unwrap()
            .into_samples::<i16>()
            .map(|s| f32::from(s.unwrap()) / f32::from(i16::MAX))
            .collect();
        let mut padded = vec![0.0; 16_000 * 2];
        padded.extend(speech);
        padded.extend(vec![0.0; 16_000 * 2]);
        let transcript = transcribe(&mut engine, &padded, &options).unwrap();
        println!(
            "speech: {:?} ({} ms)",
            transcript.text, transcript.duration_ms
        );
        assert!(transcript.text.to_lowercase().contains("quick brown fox"));
    }
}
