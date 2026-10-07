//! Putting transcribed text wherever the cursor is, in any app.
//!
//! Strategy (from Sotto):
//! 1. Wait until the user has physically released every modifier, or the
//!    target app would see Ctrl/Win/Alt held and run shortcuts instead.
//! 2. Type with Unicode key events: works where paste is blocked and in any
//!    language.
//! 3. For long text, or if typing fails, paste through the clipboard and
//!    restore what was there before.
//! 4. If injection is impossible (an elevated app), leave the text on the
//!    clipboard and say so.

#[cfg(not(windows))]
mod stub;
#[cfg(windows)]
mod windows;

use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;

/// How long to wait for the user to let go of modifier keys.
const MODIFIER_TIMEOUT: Duration = Duration::from_millis(1500);
/// Longer text is pasted rather than typed, in `Auto` mode.
const TYPE_LIMIT: usize = 300;
/// Time for the target app to read the clipboard before it's restored.
const PASTE_SETTLE: Duration = Duration::from_millis(200);

pub use crate::store::settings::InsertMethod;
use crate::store::settings::Settings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "camelCase")]
pub enum InsertResult {
    Typed,
    Pasted,
    /// Auto-paste is off: the text is on the clipboard.
    CopiedOnly,
    /// The focused app runs as administrator and Yap doesn't, so Windows
    /// blocks typing into it. The text is on the clipboard.
    CopiedElevated,
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InsertOptions {
    pub method: InsertMethod,
    pub auto_paste: bool,
    pub restore_clipboard: bool,
    pub trailing_space: bool,
}

impl InsertOptions {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            method: settings.insert_method,
            auto_paste: settings.auto_paste,
            restore_clipboard: settings.restore_clipboard,
            trailing_space: settings.trailing_space,
        }
    }
}

/// Sends keystrokes to the focused app. One implementation per OS.
pub trait Injector {
    fn type_text(&self, text: &str) -> Result<(), String>;
    /// Sends the paste shortcut (Ctrl+V or Cmd+V).
    fn paste_shortcut(&self) -> Result<(), String>;
    /// Blocks until no modifier key is held, or `timeout` passes.
    fn wait_for_modifiers_released(&self, timeout: Duration);
    /// True when the focused app runs with higher privileges than Yap.
    fn foreground_is_elevated(&self) -> bool;
}

#[cfg(windows)]
fn platform() -> impl Injector {
    windows::WindowsInjector
}

#[cfg(not(windows))]
fn platform() -> impl Injector {
    stub::StubInjector
}

/// Inserts `text` into the focused app. Blocks for as long as typing takes,
/// so call it from a background thread.
pub fn insert(text: &str, options: InsertOptions) -> InsertResult {
    let text = prepare(text, options.trailing_space);
    if text.is_empty() {
        return InsertResult::Typed;
    }
    if !options.auto_paste {
        return copy_only(&text, InsertResult::CopiedOnly);
    }

    let injector = platform();
    injector.wait_for_modifiers_released(MODIFIER_TIMEOUT);
    if injector.foreground_is_elevated() {
        return copy_only(&text, InsertResult::CopiedElevated);
    }

    let paste_first = match options.method {
        InsertMethod::Type => false,
        InsertMethod::Paste => true,
        // Pasting long text is faster, but only when the clipboard can be
        // put back; otherwise type so the user's clipboard isn't lost.
        InsertMethod::Auto => text.chars().count() > TYPE_LIMIT && clipboard::can_restore(),
    };

    if !paste_first {
        match injector.type_text(&text) {
            Ok(()) => return InsertResult::Typed,
            Err(error) => log::warn!("typing failed ({error}); pasting instead"),
        }
    }
    match paste(&injector, &text, options.restore_clipboard) {
        Ok(()) => InsertResult::Pasted,
        Err(error) => {
            log::warn!("pasting failed: {error}");
            copy_only(&text, InsertResult::Failed(error))
        }
    }
}

/// Normalizes line endings and adds the optional trailing space.
fn prepare(text: &str, trailing_space: bool) -> String {
    let mut text = text.replace("\r\n", "\n").replace('\r', "\n");
    if trailing_space && !text.is_empty() && !text.ends_with(char::is_whitespace) {
        text.push(' ');
    }
    text
}

fn copy_only(text: &str, result: InsertResult) -> InsertResult {
    match clipboard::set_text(text) {
        Ok(()) => result,
        Err(error) => InsertResult::Failed(error),
    }
}

fn paste(injector: &impl Injector, text: &str, restore: bool) -> Result<(), String> {
    let saved = restore.then(clipboard::save).flatten();
    clipboard::set_text(text)?;
    injector.paste_shortcut()?;
    sleep(PASTE_SETTLE);
    if let Some(saved) = saved {
        if let Err(error) = clipboard::restore(saved) {
            log::warn!("couldn't restore the clipboard: {error}");
        }
    }
    Ok(())
}

mod clipboard {
    //! Clipboard access via `arboard`. Text and images can be saved and
    //! restored; other formats (such as copied files) can't.

    pub enum Saved {
        Text(String),
        Image(arboard::ImageData<'static>),
        Empty,
    }

    fn open() -> Result<arboard::Clipboard, String> {
        arboard::Clipboard::new().map_err(|e| e.to_string())
    }

    pub fn set_text(text: &str) -> Result<(), String> {
        open()?.set_text(text).map_err(|e| e.to_string())
    }

    /// The current clipboard contents, if they can be put back later.
    pub fn save() -> Option<Saved> {
        let mut clipboard = open().ok()?;
        if let Ok(text) = clipboard.get_text() {
            return Some(Saved::Text(text));
        }
        if let Ok(image) = clipboard.get_image() {
            return Some(Saved::Image(image.to_owned_img()));
        }
        (!has_other_formats()).then_some(Saved::Empty)
    }

    pub fn can_restore() -> bool {
        save().is_some()
    }

    pub fn restore(saved: Saved) -> Result<(), String> {
        let mut clipboard = open()?;
        match saved {
            Saved::Text(text) => clipboard.set_text(text),
            Saved::Image(image) => clipboard.set_image(image),
            Saved::Empty => clipboard.clear(),
        }
        .map_err(|e| e.to_string())
    }

    /// True when the clipboard holds something that's neither text nor an
    /// image (files, rich formats only), which we can't save.
    fn has_other_formats() -> bool {
        #[cfg(windows)]
        {
            super::windows::clipboard_has_data()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepares_text() {
        assert_eq!(prepare("a\r\nb\rc", false), "a\nb\nc");
        assert_eq!(prepare("Hello.", true), "Hello. ");
        assert_eq!(prepare("Hello. ", true), "Hello. ");
        assert_eq!(prepare("", true), "");
    }

    #[test]
    fn results_serialize_for_the_frontend() {
        let value = serde_json::to_value(InsertResult::CopiedElevated).unwrap();
        assert_eq!(value["kind"], "copiedElevated");
        let value = serde_json::to_value(InsertResult::Failed("x".into())).unwrap();
        assert_eq!(value["message"], "x");
    }
}

#[cfg(test)]
mod real_tests {
    use super::*;

    /// Focus a text field (e.g. Notepad) within 4 seconds of starting:
    /// `cargo test -- --ignored inserts_into_the_focused_app --nocapture`
    #[test]
    #[ignore = "types into whatever app has focus"]
    fn inserts_into_the_focused_app() {
        sleep(Duration::from_secs(4));
        let options = InsertOptions {
            method: InsertMethod::Type,
            auto_paste: true,
            restore_clipboard: true,
            trailing_space: false,
        };
        let typed = insert("Typed: café naïve – नमस्ते مرحبا 😀\n", options);
        println!("typed: {typed:?}");
        assert_eq!(typed, InsertResult::Typed);

        clipboard::set_text("original clipboard").unwrap();
        let long = format!("Pasted: {}\n", "lorem ipsum ".repeat(30));
        let pasted = insert(
            &long,
            InsertOptions {
                method: InsertMethod::Auto,
                ..options
            },
        );
        println!("long text: {pasted:?}");
        assert_eq!(pasted, InsertResult::Pasted);
        let restored = arboard::Clipboard::new().unwrap().get_text().unwrap();
        println!("clipboard after paste: {restored:?}");
        assert_eq!(restored, "original clipboard");
    }
}
