//! A loaded Whisper model that turns 16 kHz mono audio into text.
//!
//! `Engine` is synchronous and slow (seconds per call). Only the worker
//! thread in `manager.rs` uses it, never the main thread or an async task.

use std::path::Path;
use std::time::Instant;

use serde::Serialize;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use super::SttError;

/// Segments Whisper itself rates as more likely silence than speech.
const MAX_NO_SPEECH_PROBABILITY: f32 = 0.6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    /// Let Whisper detect the spoken language.
    Auto,
    /// An ISO 639-1 code such as `"en"`.
    Fixed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscribeOptions {
    pub language: Language,
    /// Translate the speech into English.
    pub translate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub text: String,
    /// Detected (or forced) language code, when Whisper ran.
    pub language: Option<String>,
    /// How long transcription took.
    pub duration_ms: u64,
    pub segments: Vec<Segment>,
}

pub struct Engine {
    // The context must outlive the state; field order drops the state first.
    state: WhisperState,
    context: WhisperContext,
    threads: i32,
}

impl Engine {
    pub fn load(path: &Path, threads: usize) -> Result<Self, SttError> {
        let path_text = path
            .to_str()
            .ok_or_else(|| SttError::Load("the model path isn't valid text".into()))?;
        let mut context_params = WhisperContextParameters::default();
        // Faster on CPU, with the same output.
        context_params.flash_attn(true);
        let context = WhisperContext::new_with_params(path_text, context_params)
            .map_err(|e| SttError::Load(e.to_string()))?;
        let state = context
            .create_state()
            .map_err(|e| SttError::Load(e.to_string()))?;
        Ok(Self {
            state,
            context,
            threads: i32::try_from(threads).unwrap_or(4),
        })
    }

    pub fn is_multilingual(&self) -> bool {
        self.context.is_multilingual()
    }

    pub fn transcribe(
        &mut self,
        samples: &[f32],
        options: &TranscribeOptions,
    ) -> Result<Transcript, SttError> {
        let started = Instant::now();
        let language = match (&options.language, self.is_multilingual()) {
            (_, false) => "en",
            (Language::Auto, true) => "auto",
            (Language::Fixed(code), true) => code.as_str(),
        };

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(self.threads);
        params.set_language(Some(language));
        params.set_translate(options.translate && self.is_multilingual());
        // Each dictation stands alone; earlier text must not steer this one.
        params.set_no_context(true);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        if let Some(audio_ctx) = audio_context(samples.len()) {
            params.set_audio_ctx(audio_ctx);
        }

        self.state
            .full(params, samples)
            .map_err(|e| SttError::Transcribe(e.to_string()))?;

        let mut segments = Vec::new();
        for segment in self.state.as_iter() {
            if segment.no_speech_probability() > MAX_NO_SPEECH_PROBABILITY {
                continue;
            }
            let text = segment
                .to_str_lossy()
                .map_err(|e| SttError::Transcribe(e.to_string()))?;
            segments.push(Segment {
                // Whisper timestamps are in centiseconds.
                start_ms: segment.start_timestamp() * 10,
                end_ms: segment.end_timestamp() * 10,
                text: text.into_owned(),
            });
        }

        let detected = whisper_rs::get_lang_str(self.state.full_lang_id_from_state());
        Ok(Transcript {
            text: super::filter::join_segments(&segments),
            language: detected.map(str::to_string),
            duration_ms: started.elapsed().as_millis() as u64,
            segments,
        })
    }
}

/// Whisper always encodes a 30-second window (1500 frames, 50 per second),
/// padding short clips with silence. Most dictations are a few seconds, so
/// shrinking the window to the clip (plus a margin) skips most of that work.
/// Returns `None` for clips long enough to need the full window.
fn audio_context(samples: usize) -> Option<i32> {
    const FULL: usize = 1500;
    const MARGIN: usize = 128;
    let frames = samples.div_ceil(16_000 / 50) + MARGIN;
    // Round up to a multiple of 64, which the encoder handles best.
    let frames = frames.div_ceil(64) * 64;
    (frames < FULL).then_some(frames as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_context_fits_short_clips() {
        assert_eq!(audio_context(16_000 * 3), Some(320)); // 150 + 128 → 320
        assert_eq!(audio_context(16_000 * 20), Some(1152));
        assert_eq!(audio_context(16_000 * 28), None);
        assert_eq!(audio_context(16_000 * 60), None);
    }

    /// Benchmark and accuracy check against a real model and WAV file.
    /// `YAP_MODEL=path/to/ggml.bin YAP_WAV=path/to/16k.wav \
    ///  cargo test --release -- --ignored bench_transcription --nocapture`
    #[test]
    #[ignore = "needs a downloaded model and a WAV file"]
    fn bench_transcription() {
        let model = std::env::var("YAP_MODEL").expect("set YAP_MODEL");
        let wav = std::env::var("YAP_WAV").expect("set YAP_WAV");
        let samples: Vec<f32> = hound::WavReader::open(wav)
            .unwrap()
            .into_samples::<i16>()
            .map(|s| f32::from(s.unwrap()) / f32::from(i16::MAX))
            .collect();

        let threads = std::env::var("YAP_THREADS")
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or_else(super::super::default_threads);
        let load_started = Instant::now();
        let mut engine = Engine::load(Path::new(&model), threads).unwrap();
        let load_ms = load_started.elapsed().as_millis();
        let options = TranscribeOptions {
            language: Language::Auto,
            translate: false,
        };
        // Warm up once, then time a second run.
        engine.transcribe(&samples, &options).unwrap();
        let transcript = engine.transcribe(&samples, &options).unwrap();
        println!(
            "model {model}\naudio {:.1}s, {threads} threads, load {load_ms} ms, transcribe {} ms\nlanguage {:?}\ntext {:?}",
            samples.len() as f32 / 16_000.0,
            transcript.duration_ms,
            transcript.language,
            transcript.text
        );
        assert!(!transcript.text.is_empty());
    }
}
