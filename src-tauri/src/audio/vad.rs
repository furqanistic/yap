//! Silence gate: decides whether a recording contains speech before Whisper
//! sees it. Whisper invents text for silence and noise ("Thank you."), so
//! audio without speech never reaches it.
//!
//! Each 30 ms frame is scored on loudness and spectral flatness. Noise spreads
//! its energy evenly across frequencies (flat spectrum); voiced speech puts it
//! in a few harmonics (peaky spectrum). A later upgrade can swap in Silero VAD
//! behind the same two functions.

use realfft::RealFftPlanner;

use super::TARGET_SAMPLE_RATE;

const FRAME: usize = (TARGET_SAMPLE_RATE as usize * 30) / 1000;
/// Frames quieter than this (about -46 dBFS) are never speech.
const MIN_RMS: f32 = 0.005;
/// A frame must also be this many times louder than the background level.
const ABOVE_FLOOR: f32 = 2.0;
/// The background estimate is capped, so a clip that is all speech (no
/// pauses) isn't mistaken for loud noise.
const MAX_FLOOR: f32 = 0.01;
/// Speech frames have a peaky spectrum; noise sits near 0.5 and above.
const MAX_FLATNESS: f32 = 0.3;
/// At least this much speech (in frames) for the clip to count: 210 ms.
const MIN_SPEECH_FRAMES: usize = 7;
/// Audio kept around the detected speech when trimming.
const PAD_BEFORE: usize = FRAME * 7; // 210 ms
const PAD_AFTER: usize = FRAME * 10; // 300 ms
/// Only this band matters for speech (Hz).
const BAND: (f32, f32) = (100.0, 4000.0);

/// True if the 16 kHz mono `samples` contain enough speech to transcribe.
pub fn has_speech(samples: &[f32]) -> bool {
    speech_frames(samples).iter().filter(|&&s| s).count() >= MIN_SPEECH_FRAMES
}

/// The part of `samples` from just before the first speech to just after
/// the last. Returns an empty slice when there's no speech.
pub fn trim_silence(samples: &[f32]) -> &[f32] {
    let frames = speech_frames(samples);
    let (Some(first), Some(last)) = (
        frames.iter().position(|&s| s),
        frames.iter().rposition(|&s| s),
    ) else {
        return &[];
    };
    let start = (first * FRAME).saturating_sub(PAD_BEFORE);
    let end = ((last + 1) * FRAME + PAD_AFTER).min(samples.len());
    &samples[start..end]
}

/// Whether each 30 ms frame looks like speech.
fn speech_frames(samples: &[f32]) -> Vec<bool> {
    let (frames, _) = samples.as_chunks::<FRAME>();
    if frames.is_empty() {
        return Vec::new();
    }
    let energies: Vec<f32> = frames.iter().map(|f| super::capture::rms(f)).collect();
    let floor = percentile(&energies, 0.1).min(MAX_FLOOR);
    let threshold = MIN_RMS.max(floor * ABOVE_FLOOR);

    let mut flatness = SpectralFlatness::new();
    frames
        .iter()
        .zip(&energies)
        .map(|(frame, &energy)| energy >= threshold && flatness.of(frame) <= MAX_FLATNESS)
        .collect()
}

fn percentile(values: &[f32], p: f32) -> f32 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    sorted[((sorted.len() - 1) as f32 * p) as usize]
}

/// Geometric mean over arithmetic mean of the power spectrum, within the
/// speech band: near 1 for white noise, near 0 for a pure tone.
struct SpectralFlatness {
    fft: std::sync::Arc<dyn realfft::RealToComplex<f32>>,
    input: Vec<f32>,
    output: Vec<realfft::num_complex::Complex<f32>>,
    window: Vec<f32>,
}

impl SpectralFlatness {
    fn new() -> Self {
        let fft = RealFftPlanner::<f32>::new().plan_fft_forward(FRAME);
        let window = (0..FRAME)
            .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (FRAME - 1) as f32).cos())
            .collect();
        Self {
            input: fft.make_input_vec(),
            output: fft.make_output_vec(),
            fft,
            window,
        }
    }

    fn of(&mut self, frame: &[f32]) -> f32 {
        for ((slot, sample), weight) in self.input.iter_mut().zip(frame).zip(&self.window) {
            *slot = sample * weight;
        }
        if self.fft.process(&mut self.input, &mut self.output).is_err() {
            return 1.0;
        }
        let bin_hz = TARGET_SAMPLE_RATE as f32 / FRAME as f32;
        let (low, high) = ((BAND.0 / bin_hz) as usize, (BAND.1 / bin_hz) as usize);
        let powers: Vec<f32> = self.output[low..=high.min(self.output.len() - 1)]
            .iter()
            .map(|c| c.norm_sqr() + 1e-12)
            .collect();
        let log_mean = powers.iter().map(|p| p.ln()).sum::<f32>() / powers.len() as f32;
        let mean = powers.iter().sum::<f32>() / powers.len() as f32;
        log_mean.exp() / mean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = TARGET_SAMPLE_RATE as f32;

    /// Deterministic pseudo-random noise in -1..1.
    fn noise(len: usize, seed: u32) -> impl Iterator<Item = f32> {
        let mut state = seed;
        (0..len).map(move |_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state as f32 / u32::MAX as f32) * 2.0 - 1.0
        })
    }

    /// A voiced, syllable-like signal: a 150 Hz voice with harmonics whose
    /// loudness rises and falls four times a second.
    fn speech_like(seconds: f32) -> Vec<f32> {
        (0..(RATE * seconds) as usize)
            .map(|i| {
                let t = i as f32 / RATE;
                let voice: f32 = (1..=8)
                    .map(|h| (std::f32::consts::TAU * 150.0 * h as f32 * t).sin() / h as f32)
                    .sum();
                let syllables = 0.5 + 0.5 * (std::f32::consts::TAU * 4.0 * t).sin();
                voice * syllables * 0.08
            })
            .collect()
    }

    fn silence(seconds: f32) -> Vec<f32> {
        vec![0.0; (RATE * seconds) as usize]
    }

    #[test]
    fn silence_has_no_speech() {
        assert!(!has_speech(&silence(2.0)));
        assert!(!has_speech(&[]));
        assert!(trim_silence(&silence(1.0)).is_empty());
    }

    #[test]
    fn white_noise_has_no_speech() {
        let quiet: Vec<f32> = noise(32_000, 7).map(|s| s * 0.003).collect();
        let loud: Vec<f32> = noise(32_000, 9).map(|s| s * 0.3).collect();
        assert!(!has_speech(&quiet));
        assert!(!has_speech(&loud));
    }

    #[test]
    fn steady_fan_noise_has_no_speech() {
        // Low-passed noise, like a fan or air conditioner.
        let mut last = 0.0;
        let fan: Vec<f32> = noise(48_000, 3)
            .map(|s| {
                last = 0.9 * last + 0.1 * s;
                last * 0.05
            })
            .collect();
        assert!(!has_speech(&fan));
    }

    #[test]
    fn speech_is_detected_even_without_pauses() {
        assert!(has_speech(&speech_like(1.5)));
    }

    #[test]
    fn trimming_keeps_the_speech_and_some_padding() {
        let mut clip = silence(1.0);
        clip.extend(speech_like(1.0));
        clip.extend(silence(1.5));
        let trimmed = trim_silence(&clip);
        let seconds = trimmed.len() as f32 / RATE;
        assert!((1.0..1.7).contains(&seconds), "{seconds}s");
    }

    #[test]
    fn speech_over_quiet_noise_is_detected() {
        let speech = speech_like(1.0);
        let noisy: Vec<f32> = speech
            .iter()
            .zip(noise(speech.len(), 11))
            .map(|(s, n)| s + n * 0.004)
            .collect();
        assert!(has_speech(&noisy));
    }
}
