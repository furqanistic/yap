//! Windows injection: `SendInput` with Unicode key events, Ctrl+V for
//! pasting, and an integrity-level check for elevated (admin) windows.

use std::thread::sleep;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, TokenIntegrityLevel,
    TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
};
use windows::Win32::System::DataExchange::CountClipboardFormats;
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RETURN,
    VK_RWIN, VK_SHIFT, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::Injector;

/// Characters per `SendInput` batch, with a short pause in between so slow
/// apps (Electron, terminals) keep up.
const CHUNK: usize = 24;
const CHUNK_PAUSE: Duration = Duration::from_millis(8);

pub struct WindowsInjector;

impl Injector for WindowsInjector {
    fn type_text(&self, text: &str) -> Result<(), String> {
        let events = text_to_events(text);
        for chunk in events.chunks(CHUNK * 2) {
            let inputs: Vec<INPUT> = chunk.iter().map(|e| e.to_input()).collect();
            send(&inputs)?;
            sleep(CHUNK_PAUSE);
        }
        Ok(())
    }

    fn paste_shortcut(&self) -> Result<(), String> {
        let v = VIRTUAL_KEY(u16::from(b'V'));
        let inputs = [
            KeyEvent::Virtual(VK_CONTROL, false).to_input(),
            KeyEvent::Virtual(v, false).to_input(),
            KeyEvent::Virtual(v, true).to_input(),
            KeyEvent::Virtual(VK_CONTROL, true).to_input(),
        ];
        send(&inputs)
    }

    fn wait_for_modifiers_released(&self, timeout: Duration) {
        let started = Instant::now();
        let held = || {
            [VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN]
                .iter()
                .any(|vk| unsafe { GetAsyncKeyState(i32::from(vk.0)) } < 0)
        };
        while held() && started.elapsed() < timeout {
            sleep(Duration::from_millis(10));
        }
    }

    fn foreground_is_elevated(&self) -> bool {
        unsafe {
            let window = GetForegroundWindow();
            let mut process_id = 0;
            GetWindowThreadProcessId(window, Some(&mut process_id));
            if process_id == 0 {
                return false;
            }
            let Some(ours) = integrity_level(GetCurrentProcess()) else {
                return false;
            };
            match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
                Ok(process) => {
                    let theirs = integrity_level(process);
                    let _ = CloseHandle(process);
                    // Unreadable usually means more privileged than us.
                    theirs.is_none_or(|level| level > ours)
                }
                Err(_) => true,
            }
        }
    }
}

/// The process's integrity level RID (e.g. 0x2000 medium, 0x3000 high).
unsafe fn integrity_level(process: HANDLE) -> Option<u32> {
    let mut token = HANDLE::default();
    unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) }.ok()?;
    let mut buffer = vec![0u8; 256];
    let mut length = 0;
    let read = unsafe {
        GetTokenInformation(
            token,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr().cast()),
            buffer.len() as u32,
            &mut length,
        )
    };
    let _ = unsafe { CloseHandle(token) };
    read.ok()?;
    // SAFETY: GetTokenInformation filled the buffer with a TOKEN_MANDATORY_LABEL.
    let label = unsafe { &*(buffer.as_ptr() as *const TOKEN_MANDATORY_LABEL) };
    let sid = label.Label.Sid;
    unsafe {
        let count = *GetSidSubAuthorityCount(sid);
        Some(*GetSidSubAuthority(sid, u32::from(count.checked_sub(1)?)))
    }
}

/// True when the clipboard holds anything at all, in any format.
pub fn clipboard_has_data() -> bool {
    unsafe { CountClipboardFormats() > 0 }
}

fn send(inputs: &[INPUT]) -> Result<(), String> {
    let sent = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize == inputs.len() {
        Ok(())
    } else {
        Err(format!(
            "Windows accepted {sent} of {} key events",
            inputs.len()
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyEvent {
    /// A UTF-16 code unit, sent as a Unicode key event.
    Unicode(u16, bool),
    Virtual(VIRTUAL_KEY, bool),
}

impl KeyEvent {
    fn to_input(self) -> INPUT {
        let (vk, scan, flags) = match self {
            KeyEvent::Unicode(unit, up) => (
                VIRTUAL_KEY(0),
                unit,
                KEYEVENTF_UNICODE
                    | if up {
                        KEYEVENTF_KEYUP
                    } else {
                        KEYBD_EVENT_FLAGS(0)
                    },
            ),
            KeyEvent::Virtual(vk, up) => (
                vk,
                0,
                if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
            ),
        };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    ..Default::default()
                },
            },
        }
    }
}

/// Key presses and releases that type `text`. Newlines become Enter and
/// tabs become Tab; characters above U+FFFF (emoji) are sent as surrogate
/// pairs, which apps reassemble.
fn text_to_events(text: &str) -> Vec<KeyEvent> {
    let mut events = Vec::with_capacity(text.len() * 2);
    for ch in text.chars() {
        match ch {
            '\n' => events.extend([
                KeyEvent::Virtual(VK_RETURN, false),
                KeyEvent::Virtual(VK_RETURN, true),
            ]),
            '\r' => {}
            '\t' => events.extend([
                KeyEvent::Virtual(VK_TAB, false),
                KeyEvent::Virtual(VK_TAB, true),
            ]),
            _ => {
                let mut units = [0u16; 2];
                for &unit in ch.encode_utf16(&mut units).iter() {
                    events.push(KeyEvent::Unicode(unit, false));
                    events.push(KeyEvent::Unicode(unit, true));
                }
            }
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_becomes_press_and_release() {
        assert_eq!(
            text_to_events("hi"),
            vec![
                KeyEvent::Unicode(u16::from(b'h'), false),
                KeyEvent::Unicode(u16::from(b'h'), true),
                KeyEvent::Unicode(u16::from(b'i'), false),
                KeyEvent::Unicode(u16::from(b'i'), true),
            ]
        );
    }

    #[test]
    fn emoji_is_sent_as_a_surrogate_pair() {
        let events = text_to_events("\u{1F600}");
        assert_eq!(
            events,
            vec![
                KeyEvent::Unicode(0xD83D, false),
                KeyEvent::Unicode(0xD83D, true),
                KeyEvent::Unicode(0xDE00, false),
                KeyEvent::Unicode(0xDE00, true),
            ]
        );
    }

    #[test]
    fn newlines_and_tabs_become_keys() {
        let events = text_to_events("a\r\n\tb");
        assert!(events.contains(&KeyEvent::Virtual(VK_RETURN, false)));
        assert!(events.contains(&KeyEvent::Virtual(VK_TAB, false)));
        assert_eq!(events.len(), 8);
    }

    #[test]
    fn non_latin_scripts_are_single_units() {
        // Hindi and Arabic letters are in the Basic Multilingual Plane.
        assert_eq!(text_to_events("\u{0928}\u{0645}").len(), 4);
    }
}
