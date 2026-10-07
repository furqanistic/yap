//! Cleaning Whisper's output and dropping text it invented.

use super::engine::Segment;

/// Phrases Whisper tends to produce from silence or noise, normalized
/// (lowercase, letters and spaces only).
const HALLUCINATIONS: &[&str] = &[
    "thank you",
    "thank you very much",
    "thanks",
    "thanks for watching",
    "thank you for watching",
    "thanks for listening",
    "thank you for listening",
    "please subscribe",
    "like and subscribe",
    "subtitles by the amaraorg community",
    "you",
    "bye",
    "bye bye",
    "okay",
    "so",
];

/// Above this RMS the audio was clearly loud, so even a stock phrase like
/// "Thank you." is probably what the person said.
const QUIET_RMS: f32 = 0.02;

/// Joins segments into one line of text without Whisper's annotations.
pub fn join_segments(segments: &[Segment]) -> String {
    let joined = segments
        .iter()
        .map(|segment| strip_annotations(&segment.text))
        .collect::<Vec<_>>()
        .join(" ");
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Removes `[BLANK_AUDIO]`, `(music)`, `*laughs*` and similar.
fn strip_annotations(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut closing: Option<char> = None;
    for ch in text.chars() {
        match closing {
            Some(close) if ch == close => closing = None,
            Some(_) => {}
            None => match ch {
                '[' => closing = Some(']'),
                '(' => closing = Some(')'),
                '*' => closing = Some('*'),
                _ => out.push(ch),
            },
        }
    }
    out
}

fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// True when `text` is a known hallucination and the audio was quiet, so
/// it's almost certainly invented rather than spoken.
pub fn is_hallucination(text: &str, audio_rms: f32) -> bool {
    let normalized = normalize(text);
    if normalized.is_empty() {
        return true;
    }
    audio_rms < QUIET_RMS && HALLUCINATIONS.contains(&normalized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(text: &str) -> Segment {
        Segment {
            start_ms: 0,
            end_ms: 0,
            text: text.into(),
        }
    }

    #[test]
    fn joins_and_strips_annotations() {
        let text = join_segments(&[
            segment(" [BLANK_AUDIO] Hello there."),
            segment("  (music) How are you? *laughs*"),
        ]);
        assert_eq!(text, "Hello there. How are you?");
    }

    #[test]
    fn known_phrases_on_quiet_audio_are_hallucinations() {
        assert!(is_hallucination("Thank you.", 0.004));
        assert!(is_hallucination(" Thanks for watching! ", 0.01));
        assert!(is_hallucination("", 0.5));
        assert!(is_hallucination(" ... ", 0.5));
    }

    #[test]
    fn real_speech_is_kept() {
        assert!(!is_hallucination("Thank you.", 0.08), "loud audio");
        assert!(!is_hallucination("Thank you for the report.", 0.004));
        assert!(!is_hallucination("Send it on Friday.", 0.004));
    }
}
