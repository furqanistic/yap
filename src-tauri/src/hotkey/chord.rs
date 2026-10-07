//! Keys, chords and the hold-to-talk state machine. Pure logic with no OS
//! calls, so it's tested on its own.

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

/// A physical key. Left and right modifiers are the same key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Ctrl,
    Alt,
    Shift,
    /// Windows key, or Command on macOS.
    Win,
    /// A letter or digit, uppercase: 'A'..='Z', '0'..='9'.
    Char(char),
    /// F1 to F24.
    F(u8),
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    CapsLock,
    Minus,
    Equal,
    Comma,
    Period,
    Slash,
    Semicolon,
    Quote,
    BracketLeft,
    BracketRight,
    Backslash,
    Backquote,
}

impl Key {
    pub fn is_modifier(self) -> bool {
        matches!(self, Key::Ctrl | Key::Alt | Key::Shift | Key::Win)
    }

    /// Parses one key label as recorded by the UI's `ShortcutInput`.
    pub fn from_label(label: &str) -> Option<Key> {
        let key = match label {
            "Ctrl" | "Control" | "\u{2303}" => Key::Ctrl,
            "Alt" | "Option" | "\u{2325}" => Key::Alt,
            "Shift" | "\u{21e7}" => Key::Shift,
            "Win" | "Meta" | "Cmd" | "Command" | "Super" | "\u{2318}" => Key::Win,
            "Space" | " " => Key::Space,
            "Enter" => Key::Enter,
            "Tab" => Key::Tab,
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "Insert" => Key::Insert,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "ArrowUp" => Key::ArrowUp,
            "ArrowDown" => Key::ArrowDown,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "CapsLock" => Key::CapsLock,
            // Shifted labels map to the same physical key.
            "-" | "_" => Key::Minus,
            "=" | "+" => Key::Equal,
            "," | "<" => Key::Comma,
            "." | ">" => Key::Period,
            "/" | "?" => Key::Slash,
            ";" | ":" => Key::Semicolon,
            "'" | "\"" => Key::Quote,
            "[" | "{" => Key::BracketLeft,
            "]" | "}" => Key::BracketRight,
            "\\" | "|" => Key::Backslash,
            "`" | "~" => Key::Backquote,
            _ => {
                if let Some(number) = label.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()) {
                    return (1..=24).contains(&number).then_some(Key::F(number));
                }
                let mut chars = label.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) if c.is_ascii_alphanumeric() => {
                        Key::Char(c.to_ascii_uppercase())
                    }
                    _ => return None,
                }
            }
        };
        Some(key)
    }
}

/// The set of keys that must be held together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    keys: BTreeSet<Key>,
}

impl Chord {
    /// Parses the labels stored in settings, e.g. `["Ctrl", "Win"]`.
    pub fn parse<S: AsRef<str>>(labels: &[S]) -> Result<Self, String> {
        let mut keys = BTreeSet::new();
        for label in labels {
            let label = label.as_ref();
            let key = Key::from_label(label).ok_or_else(|| format!("unknown key \"{label}\""))?;
            keys.insert(key);
        }
        if keys.is_empty() {
            return Err("the shortcut is empty".into());
        }
        Ok(Self { keys })
    }

    pub fn contains(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordEvent {
    /// Every key of the chord is held.
    Down,
    /// A chord key was released after a real hold.
    Up,
    /// Released too quickly to be intentional.
    Cancelled,
}

/// Holds shorter than this are treated as accidental.
pub const MIN_HOLD: Duration = Duration::from_millis(200);

/// Turns a stream of key presses and releases into chord events.
///
/// `Down` fires when the last chord key goes down while no other keys are
/// held (so Ctrl+Space doesn't fire on Ctrl+Shift+Space). Once active, extra
/// keys are ignored. Releasing any chord key ends the hold.
#[derive(Debug)]
pub struct ChordMatcher {
    chord: Option<Chord>,
    held: BTreeSet<Key>,
    active_since: Option<Instant>,
}

impl ChordMatcher {
    pub fn new(chord: Option<Chord>) -> Self {
        Self {
            chord,
            held: BTreeSet::new(),
            active_since: None,
        }
    }

    pub fn chord(&self) -> Option<&Chord> {
        self.chord.as_ref()
    }

    /// Replaces the chord. An active hold ends without an event.
    pub fn set_chord(&mut self, chord: Option<Chord>) {
        self.chord = chord;
        self.active_since = None;
    }

    pub fn is_active(&self) -> bool {
        self.active_since.is_some()
    }

    pub fn key_down(&mut self, key: Key, now: Instant) -> Option<ChordEvent> {
        // Auto-repeat sends more key-downs for a held key.
        if !self.held.insert(key) || self.active_since.is_some() {
            return None;
        }
        let chord = self.chord.as_ref()?;
        let exact = self.held.len() == chord.keys.len() && chord.keys.is_subset(&self.held);
        if exact {
            self.active_since = Some(now);
            return Some(ChordEvent::Down);
        }
        None
    }

    pub fn key_up(&mut self, key: Key, now: Instant) -> Option<ChordEvent> {
        self.held.remove(&key);
        let started = self.active_since?;
        if !self.chord.as_ref().is_some_and(|c| c.contains(key)) {
            return None;
        }
        self.active_since = None;
        Some(if now.duration_since(started) < MIN_HOLD {
            ChordEvent::Cancelled
        } else {
            ChordEvent::Up
        })
    }

    /// Forgets keys the OS says are no longer down. Key-ups can be missed,
    /// for example when Win+L locks the screen mid-press.
    pub fn forget_released(&mut self, is_down: impl Fn(Key) -> bool) -> Option<ChordEvent> {
        let stale: Vec<Key> = self.held.iter().copied().filter(|&k| !is_down(k)).collect();
        let mut event = None;
        for key in stale {
            if let Some(e) = self.key_up(key, Instant::now()) {
                event = Some(e);
            }
        }
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chord(labels: &[&str]) -> Chord {
        Chord::parse(labels).unwrap()
    }

    fn at(base: Instant, ms: u64) -> Instant {
        base + Duration::from_millis(ms)
    }

    #[test]
    fn parses_every_label_the_ui_produces() {
        let cases: &[(&str, Key)] = &[
            ("Ctrl", Key::Ctrl),
            ("Alt", Key::Alt),
            ("Shift", Key::Shift),
            ("Win", Key::Win),
            ("\u{2303}", Key::Ctrl),
            ("\u{2325}", Key::Alt),
            ("\u{21e7}", Key::Shift),
            ("\u{2318}", Key::Win),
            ("Space", Key::Space),
            ("A", Key::Char('A')),
            ("z", Key::Char('Z')),
            ("7", Key::Char('7')),
            ("F9", Key::F(9)),
            ("F24", Key::F(24)),
            ("Enter", Key::Enter),
            ("ArrowUp", Key::ArrowUp),
            ("-", Key::Minus),
            ("_", Key::Minus),
            ("/", Key::Slash),
        ];
        for (label, key) in cases {
            assert_eq!(Key::from_label(label), Some(*key), "{label}");
        }
        assert_eq!(Key::from_label("F25"), None);
        assert_eq!(Key::from_label("Unidentified"), None);
        assert!(Chord::parse::<&str>(&[]).is_err());
        assert!(Chord::parse(&["Ctrl", "Nope"]).is_err());
    }

    #[test]
    fn modifier_only_chord_goes_down_and_up() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Win"])));
        assert_eq!(m.key_down(Key::Ctrl, t), None);
        assert_eq!(m.key_down(Key::Win, at(t, 10)), Some(ChordEvent::Down));
        assert_eq!(m.key_up(Key::Win, at(t, 900)), Some(ChordEvent::Up));
        assert_eq!(m.key_up(Key::Ctrl, at(t, 950)), None);
    }

    #[test]
    fn order_of_presses_does_not_matter() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Space"])));
        assert_eq!(m.key_down(Key::Space, t), None);
        assert_eq!(m.key_down(Key::Ctrl, t), Some(ChordEvent::Down));
    }

    #[test]
    fn short_holds_are_cancelled() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Win"])));
        m.key_down(Key::Ctrl, t);
        m.key_down(Key::Win, t);
        assert_eq!(m.key_up(Key::Ctrl, at(t, 120)), Some(ChordEvent::Cancelled));
    }

    #[test]
    fn auto_repeat_and_extra_keys_are_ignored_during_a_hold() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Space"])));
        m.key_down(Key::Ctrl, t);
        assert_eq!(m.key_down(Key::Space, t), Some(ChordEvent::Down));
        assert_eq!(m.key_down(Key::Space, at(t, 30)), None);
        assert_eq!(m.key_down(Key::Shift, at(t, 40)), None);
        assert_eq!(m.key_up(Key::Shift, at(t, 50)), None);
        assert!(m.is_active());
        assert_eq!(m.key_up(Key::Space, at(t, 500)), Some(ChordEvent::Up));
    }

    #[test]
    fn a_larger_combo_does_not_trigger() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Space"])));
        m.key_down(Key::Ctrl, t);
        m.key_down(Key::Shift, t);
        assert_eq!(m.key_down(Key::Space, t), None);
    }

    #[test]
    fn can_press_again_after_release() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["F9"])));
        assert_eq!(m.key_down(Key::F(9), t), Some(ChordEvent::Down));
        assert_eq!(m.key_up(Key::F(9), at(t, 300)), Some(ChordEvent::Up));
        assert_eq!(m.key_down(Key::F(9), at(t, 400)), Some(ChordEvent::Down));
    }

    #[test]
    fn missed_key_ups_are_recovered() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["Ctrl", "Win"])));
        m.key_down(Key::Ctrl, t);
        m.key_down(Key::Win, t);
        // The Win key-up never arrived (screen locked). The OS says it's up.
        let event = m.forget_released(|key| key == Key::Ctrl);
        assert!(matches!(
            event,
            Some(ChordEvent::Up | ChordEvent::Cancelled)
        ));
        assert!(!m.is_active());
        m.key_up(Key::Ctrl, t);
        assert_eq!(m.key_down(Key::Ctrl, t), None);
        assert_eq!(m.key_down(Key::Win, t), Some(ChordEvent::Down));
    }

    #[test]
    fn changing_the_chord_ends_a_hold() {
        let t = Instant::now();
        let mut m = ChordMatcher::new(Some(chord(&["F9"])));
        m.key_down(Key::F(9), t);
        m.set_chord(Some(chord(&["F10"])));
        assert!(!m.is_active());
        assert_eq!(m.key_up(Key::F(9), t), None);
    }

    #[test]
    fn no_chord_never_fires() {
        let mut m = ChordMatcher::new(None);
        assert_eq!(m.key_down(Key::Ctrl, Instant::now()), None);
    }
}
