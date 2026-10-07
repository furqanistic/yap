//! Global hold-to-talk hotkey: "this chord is held" and "it was released",
//! anywhere in the OS, including modifier-only chords like Ctrl+Win.
//!
//! `tauri-plugin-global-shortcut` can't do modifier-only chords or report
//! releases, so each platform installs a low-level keyboard hook that feeds
//! a pure state machine (`chord::ChordMatcher`).
//!
//! ```text
//! OS key events ──▶ ChordMatcher ──▶ Down / Up / Cancelled ──▶ events
//! ```

pub mod chord;
#[cfg(not(windows))]
mod stub;
#[cfg(windows)]
mod windows;

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::Serialize;
use tokio::sync::broadcast;

use chord::{Chord, ChordEvent, ChordMatcher};

/// One implementation per OS. Windows has a real one; macOS and Linux get
/// theirs in later steps and use the stub until then.
pub trait HotkeyBackend: Send + Sync {
    /// Starts delivering key events to `matcher`, and chord events to `events`.
    fn install(
        &self,
        matcher: Arc<Mutex<ChordMatcher>>,
        events: Sender<ChordEvent>,
    ) -> Result<(), String>;

    /// Called right after `Down`, while the chord is still held.
    fn on_down(&self, _chord: &Chord) {}
}

#[cfg(windows)]
fn platform_backend() -> Box<dyn HotkeyBackend> {
    Box::new(windows::WindowsBackend)
}

#[cfg(not(windows))]
fn platform_backend() -> Box<dyn HotkeyBackend> {
    Box::new(stub::StubBackend)
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    /// True when the keyboard hook is running.
    pub installed: bool,
    /// The active chord, as stored in settings.
    pub chord: Vec<String>,
    /// Why the hook or the chord isn't working, if it isn't.
    pub error: Option<String>,
}

/// Held in Tauri managed state.
pub struct HotkeyManager {
    matcher: Arc<Mutex<ChordMatcher>>,
    status: Mutex<HotkeyStatus>,
    events: broadcast::Sender<ChordEvent>,
}

impl HotkeyManager {
    /// Installs the platform hook. `on_event` runs on a dispatcher thread for
    /// every chord event.
    pub fn start(labels: &[String], on_event: impl Fn(ChordEvent) + Send + 'static) -> Self {
        let (chord, chord_error) = match Chord::parse(labels) {
            Ok(chord) => (Some(chord), None),
            Err(error) => (None, Some(error)),
        };
        let matcher = Arc::new(Mutex::new(ChordMatcher::new(chord)));
        let (broadcast_tx, _) = broadcast::channel(16);
        let (tx, rx) = mpsc::channel::<ChordEvent>();

        let backend: Arc<dyn HotkeyBackend> = Arc::from(platform_backend());
        let installed = backend.install(matcher.clone(), tx);
        if let Err(error) = &installed {
            log::warn!("hotkey not available: {error}");
        }

        let dispatch_matcher = matcher.clone();
        let dispatch_broadcast = broadcast_tx.clone();
        std::thread::Builder::new()
            .name("yap-hotkey-events".into())
            .spawn(move || {
                for event in rx {
                    if event == ChordEvent::Down {
                        let chord = lock(&dispatch_matcher).chord().cloned();
                        if let Some(chord) = chord {
                            backend.on_down(&chord);
                        }
                    }
                    log::info!("hotkey {event:?}");
                    let _ = dispatch_broadcast.send(event);
                    on_event(event);
                }
            })
            .expect("can start the hotkey dispatcher");

        Self {
            matcher,
            status: Mutex::new(HotkeyStatus {
                installed: installed.is_ok(),
                chord: labels.to_vec(),
                error: installed.err().or(chord_error),
            }),
            events: broadcast_tx,
        }
    }

    /// Switches to a new chord right away, without a restart.
    pub fn set_chord(&self, labels: &[String]) {
        let mut status = lock(&self.status);
        if status.chord == labels {
            return;
        }
        let parsed = Chord::parse(labels);
        status.chord = labels.to_vec();
        if status.installed {
            status.error = parsed.as_ref().err().cloned();
        }
        lock(&self.matcher).set_chord(parsed.ok());
    }

    pub fn status(&self) -> HotkeyStatus {
        lock(&self.status).clone()
    }

    /// Chord events for Rust code, such as the dictation pipeline.
    #[allow(dead_code)] // The dictation pipeline subscribes in a later step.
    pub fn subscribe(&self) -> broadcast::Receiver<ChordEvent> {
        self.events.subscribe()
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}
