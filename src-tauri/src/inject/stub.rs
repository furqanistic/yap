//! Placeholder for platforms without text injection yet. Text still lands
//! on the clipboard, via the fallback in `insert`.

use std::time::Duration;

use super::Injector;

pub struct StubInjector;

impl Injector for StubInjector {
    fn type_text(&self, _text: &str) -> Result<(), String> {
        Err("typing isn't supported on this platform yet".into())
    }

    fn paste_shortcut(&self) -> Result<(), String> {
        Err("pasting isn't supported on this platform yet".into())
    }

    fn wait_for_modifiers_released(&self, _timeout: Duration) {}

    fn foreground_is_elevated(&self) -> bool {
        false
    }
}
