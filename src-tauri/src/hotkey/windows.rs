//! Windows backend: a low-level keyboard hook (`WH_KEYBOARD_LL`).
//!
//! The hook runs on its own thread with a message loop. Its callback only
//! updates the in-memory chord matcher and queues events on a channel, so it
//! returns in microseconds; a slow hook would lag every keystroke on the PC.

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HC_ACTION,
    KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use super::chord::{Chord, ChordEvent, ChordMatcher, Key};
use super::HotkeyBackend;

/// An unassigned virtual key. Pressing it while Win is down stops the Start
/// menu from opening when Win is released, as other hotkey tools do.
const VK_MASK: u16 = 0xE8;

struct Shared {
    matcher: Arc<Mutex<ChordMatcher>>,
    events: Sender<ChordEvent>,
}

static SHARED: OnceLock<Shared> = OnceLock::new();

pub struct WindowsBackend;

impl HotkeyBackend for WindowsBackend {
    fn install(
        &self,
        matcher: Arc<Mutex<ChordMatcher>>,
        events: Sender<ChordEvent>,
    ) -> Result<(), String> {
        if SHARED.set(Shared { matcher, events }).is_err() {
            return Err("the keyboard hook is already installed".into());
        }
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("yap-hotkey".into())
            .spawn(move || unsafe {
                let module = GetModuleHandleW(None).ok();
                let hook =
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), module.map(Into::into), 0);
                let installed = hook.is_ok();
                let _ = ready_tx.send(hook.map(|_| ()).map_err(|e| e.to_string()));
                if !installed {
                    return;
                }
                // The hook only runs while this thread pumps messages.
                let mut message = MSG::default();
                while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            })
            .map_err(|e| e.to_string())?;
        ready_rx
            .recv()
            .map_err(|_| "the hotkey thread stopped".to_string())?
    }

    fn on_down(&self, chord: &Chord) {
        if chord.contains(Key::Win) {
            press(VK_MASK);
        }
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, lparam points to a KBDLLHOOKSTRUCT.
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        // Ignore keys we typed ourselves (text insertion, the Win mask).
        let injected = info.flags.0 & LLKHF_INJECTED.0 != 0;
        if !injected {
            if let Some(key) = key_from_vk(info.vkCode) {
                let down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
                if handle_key(key, down) {
                    return LRESULT(1);
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Updates the matcher. Returns true to swallow the key, so the chord's
/// non-modifier key (Space in Ctrl+Space) doesn't also reach the app.
fn handle_key(key: Key, down: bool) -> bool {
    let Some(shared) = SHARED.get() else {
        return false;
    };
    // Never wait in the hook. If the lock is busy (settings changing), skip.
    let Ok(mut matcher) = shared.matcher.try_lock() else {
        return false;
    };
    let now = Instant::now();
    if down {
        if let Some(event) = matcher.forget_released(|k| k == key || is_down(k)) {
            let _ = shared.events.send(event);
        }
    }
    let event = if down {
        matcher.key_down(key, now)
    } else {
        matcher.key_up(key, now)
    };
    if let Some(event) = event {
        let _ = shared.events.send(event);
    }
    !key.is_modifier()
        && matcher.chord().is_some_and(|c| c.contains(key))
        && (matcher.is_active() || event.is_some())
}

fn is_down(key: Key) -> bool {
    vk_for(key).is_some_and(|vk| unsafe { GetAsyncKeyState(i32::from(vk)) } < 0)
}

fn press(vk: u16) {
    let input = |flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                dwFlags: flags,
                ..Default::default()
            },
        },
    };
    let inputs = [input(Default::default()), input(KEYEVENTF_KEYUP)];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Maps a Windows virtual-key code to a `Key`. Left and right modifiers
/// become the same key.
pub fn key_from_vk(vk: u32) -> Option<Key> {
    let key = match vk {
        0x10 | 0xA0 | 0xA1 => Key::Shift,
        0x11 | 0xA2 | 0xA3 => Key::Ctrl,
        0x12 | 0xA4 | 0xA5 => Key::Alt,
        0x5B | 0x5C => Key::Win,
        0x30..=0x39 | 0x41..=0x5A => Key::Char(char::from_u32(vk)?),
        0x70..=0x87 => Key::F((vk - 0x6F) as u8),
        0x20 => Key::Space,
        0x0D => Key::Enter,
        0x09 => Key::Tab,
        0x08 => Key::Backspace,
        0x2E => Key::Delete,
        0x2D => Key::Insert,
        0x24 => Key::Home,
        0x23 => Key::End,
        0x21 => Key::PageUp,
        0x22 => Key::PageDown,
        0x26 => Key::ArrowUp,
        0x28 => Key::ArrowDown,
        0x25 => Key::ArrowLeft,
        0x27 => Key::ArrowRight,
        0x14 => Key::CapsLock,
        0xBD => Key::Minus,
        0xBB => Key::Equal,
        0xBC => Key::Comma,
        0xBE => Key::Period,
        0xBF => Key::Slash,
        0xBA => Key::Semicolon,
        0xDE => Key::Quote,
        0xDB => Key::BracketLeft,
        0xDD => Key::BracketRight,
        0xDC => Key::Backslash,
        0xC0 => Key::Backquote,
        _ => return None,
    };
    Some(key)
}

/// The virtual-key code to ask the OS about. Generic codes cover both the
/// left and right modifier.
fn vk_for(key: Key) -> Option<u16> {
    let vk = match key {
        Key::Shift => 0x10,
        Key::Ctrl => 0x11,
        Key::Alt => 0x12,
        // There's no generic Win code; check the left one, which is by far
        // the most common (and on most keyboards the only one).
        Key::Win => 0x5B,
        _ => (0..=0xFE).find(|&vk| key_from_vk(vk) == Some(key))? as u16,
    };
    Some(vk)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_virtual_keys() {
        assert_eq!(key_from_vk(0xA2), Some(Key::Ctrl));
        assert_eq!(key_from_vk(0xA3), Some(Key::Ctrl));
        assert_eq!(key_from_vk(0x5C), Some(Key::Win));
        assert_eq!(key_from_vk(0x41), Some(Key::Char('A')));
        assert_eq!(key_from_vk(0x37), Some(Key::Char('7')));
        assert_eq!(key_from_vk(0x78), Some(Key::F(9)));
        assert_eq!(key_from_vk(0x87), Some(Key::F(24)));
        assert_eq!(key_from_vk(0xFF), None);
    }

    #[test]
    fn every_parsed_key_has_a_virtual_key() {
        for label in [
            "Ctrl", "Alt", "Shift", "Win", "Space", "A", "9", "F12", "-", "/",
        ] {
            let key = Key::from_label(label).unwrap();
            assert!(vk_for(key).is_some(), "{label}");
        }
    }
}

#[cfg(test)]
mod hook_tests {
    use super::*;
    use crate::hotkey::chord::Chord;

    /// Installs the real hook. Physical key presses during the test reach it;
    /// injected ones are ignored by design.
    /// `cargo test -- --ignored installs_the_keyboard_hook --nocapture`
    #[test]
    #[ignore = "installs a system-wide keyboard hook"]
    fn installs_the_keyboard_hook() {
        let chord = Chord::parse(&["F9"]).unwrap();
        let matcher = Arc::new(Mutex::new(ChordMatcher::new(Some(chord))));
        let (tx, rx) = std::sync::mpsc::channel();
        WindowsBackend.install(matcher, tx).unwrap();
        // Injected keys must not reach the matcher.
        press(0x78); // F9
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(rx.try_recv().is_err());
    }
}
