//! Speech to text, fully offline with Whisper (whisper.cpp).
//!
//! Learned from Sotto:
//! - Whisper invents text for silence and noise, so a silence gate
//!   (`audio::vad`) runs first and quiet audio never reaches the model.
//! - No initial prompt for names: it truncated output. Word fixes happen
//!   after transcription instead.
//! - The model stays loaded between dictations; loading takes seconds.

pub mod engine;
pub mod filter;
pub mod manager;

use std::fmt;

use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::models::catalog::{Languages, ModelInfo};
use crate::store::settings::{LanguageMode, Settings};
use engine::{Language, TranscribeOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SttError {
    /// The chosen model isn't downloaded.
    NoModel,
    Load(String),
    Transcribe(String),
    /// Translation needs a multilingual model.
    TranslateUnsupported,
}

impl SttError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NoModel => "noModel",
            Self::Load(_) => "load",
            Self::Transcribe(_) => "transcribe",
            Self::TranslateUnsupported => "translateUnsupported",
        }
    }
}

impl fmt::Display for SttError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoModel => write!(f, "Download a speech model to start dictating."),
            Self::Load(detail) => write!(f, "Couldn't load the speech model: {detail}"),
            Self::Transcribe(detail) => write!(f, "Transcription failed: {detail}"),
            Self::TranslateUnsupported => write!(
                f,
                "English-only models can't translate. Pick a multilingual model or turn off translation."
            ),
        }
    }
}

impl std::error::Error for SttError {}

impl Serialize for SttError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("SttError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

/// Physical cores, capped at 8. More threads stop helping beyond that.
pub fn default_threads() -> usize {
    num_cpus::get_physical().clamp(1, 8)
}

/// Maps the language settings onto Whisper options for `model`.
/// "Preferred language" behaves like auto-detect until per-language
/// preferences exist.
pub fn options_for(settings: &Settings, model: &ModelInfo) -> Result<TranscribeOptions, SttError> {
    let english_only = model.languages == Languages::EnglishOnly;
    let translate = settings.language_mode == LanguageMode::Translate;
    if translate && english_only {
        return Err(SttError::TranslateUnsupported);
    }
    Ok(TranscribeOptions {
        language: if english_only {
            Language::Fixed("en".into())
        } else {
            Language::Auto
        },
        translate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::catalog;

    #[test]
    fn language_modes_map_to_whisper_options() {
        let multilingual = catalog::find("whisper-small").unwrap();
        let english = catalog::find("whisper-small-en").unwrap();
        let mut settings = Settings::default();

        let auto = options_for(&settings, multilingual).unwrap();
        assert_eq!(auto.language, Language::Auto);
        assert!(!auto.translate);

        assert_eq!(
            options_for(&settings, english).unwrap().language,
            Language::Fixed("en".into())
        );

        settings.language_mode = LanguageMode::Preferred;
        assert_eq!(
            options_for(&settings, multilingual).unwrap().language,
            Language::Auto
        );

        settings.language_mode = LanguageMode::Translate;
        assert!(options_for(&settings, multilingual).unwrap().translate);
        assert_eq!(
            options_for(&settings, english),
            Err(SttError::TranslateUnsupported)
        );
    }

    #[test]
    fn threads_are_capped() {
        assert!((1..=8).contains(&default_threads()));
    }
}
