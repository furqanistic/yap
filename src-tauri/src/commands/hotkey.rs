//! Hold-to-talk hotkey status.

use tauri::State;

use crate::hotkey::{HotkeyManager, HotkeyStatus};

pub const DOWN_EVENT: &str = "hotkey://down";
pub const UP_EVENT: &str = "hotkey://up";
pub const CANCELLED_EVENT: &str = "hotkey://cancelled";

/// Whether the keyboard hook is installed, the active chord, and any error.
#[tauri::command]
pub fn hotkey_status(hotkey: State<'_, HotkeyManager>) -> HotkeyStatus {
    hotkey.status()
}
