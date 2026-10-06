//! Microphone capture, device listing and live levels.
//!
//! Audio is captured with cpal, mixed to mono and resampled to 16 kHz f32,
//! which is what Whisper expects. Nothing runs while Yap isn't recording or
//! testing the microphone.

pub mod capture;
pub mod device;

use std::fmt;

use serde::ser::SerializeStruct;
use serde::Serialize;

/// Sample rate Whisper expects.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioError {
    NoDevice,
    PermissionDenied,
    DeviceBusy,
    Unsupported(String),
    Other(String),
}

impl AudioError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NoDevice => "noDevice",
            Self::PermissionDenied => "permissionDenied",
            Self::DeviceBusy => "deviceBusy",
            Self::Unsupported(_) => "unsupported",
            Self::Other(_) => "other",
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDevice => write!(f, "No microphone found. Connect one and try again."),
            Self::PermissionDenied => write!(
                f,
                "Yap isn't allowed to use the microphone. Turn on microphone access in your privacy settings."
            ),
            Self::DeviceBusy => write!(f, "The microphone is busy in another app."),
            Self::Unsupported(detail) => write!(f, "This microphone isn't supported ({detail})."),
            Self::Other(detail) => write!(f, "Couldn't use the microphone: {detail}"),
        }
    }
}

impl std::error::Error for AudioError {}

/// Serialized as `{ kind, message }` for the frontend.
impl Serialize for AudioError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("AudioError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl From<cpal::Error> for AudioError {
    fn from(err: cpal::Error) -> Self {
        match err.kind() {
            cpal::ErrorKind::DeviceNotAvailable | cpal::ErrorKind::HostUnavailable => {
                Self::NoDevice
            }
            cpal::ErrorKind::PermissionDenied => Self::PermissionDenied,
            cpal::ErrorKind::DeviceBusy => Self::DeviceBusy,
            cpal::ErrorKind::UnsupportedConfig | cpal::ErrorKind::UnsupportedOperation => {
                Self::Unsupported(err.to_string())
            }
            _ => {
                // WASAPI reports a privacy block as E_ACCESSDENIED.
                let text = err.to_string();
                if text.contains("0x80070005") || text.to_lowercase().contains("access is denied") {
                    Self::PermissionDenied
                } else {
                    Self::Other(text)
                }
            }
        }
    }
}
