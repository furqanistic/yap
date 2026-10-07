//! Placeholder for platforms without a hotkey backend yet (macOS, Linux).

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use super::chord::{ChordEvent, ChordMatcher};
use super::HotkeyBackend;

pub struct StubBackend;

impl HotkeyBackend for StubBackend {
    fn install(
        &self,
        _matcher: Arc<Mutex<ChordMatcher>>,
        _events: Sender<ChordEvent>,
    ) -> Result<(), String> {
        Err("Hold-to-talk isn't supported on this platform yet.".into())
    }
}
