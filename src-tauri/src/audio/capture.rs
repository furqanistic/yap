//! Recording from a microphone into 16 kHz mono f32 samples.
//!
//! cpal streams aren't `Send` on every platform, so each recording owns a
//! dedicated thread that builds the stream, drains samples from the audio
//! callback, resamples them as they arrive, and reports levels.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, SampleFormat};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};
use serde::Serialize;

use super::{device, AudioError, TARGET_SAMPLE_RATE};

/// Level updates cover this much audio (about 30 per second).
const LEVEL_WINDOW: Duration = Duration::from_millis(33);
const RESAMPLER_CHUNK: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Level {
    pub rms: f32,
    pub peak: f32,
}

/// Why a recording ended on its own, without `stop()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EndReason {
    /// The microphone was unplugged or disabled. Audio captured so far is kept.
    DeviceLost,
    /// The recording hit the length limit.
    TimeLimit,
}

pub struct RecorderOptions {
    /// A device id, or `"default"`.
    pub device_id: String,
    /// Stops on its own after this long (watchdog).
    pub max_duration: Duration,
    /// False for a microphone test: only levels are reported.
    pub keep_audio: bool,
    pub on_level: Box<dyn FnMut(Level) + Send>,
    pub on_end: Box<dyn FnOnce(EndReason) + Send>,
}

/// A running recording. Dropping it without `stop()` also stops it.
pub struct Recorder {
    stop: Sender<()>,
    thread: Option<JoinHandle<Vec<f32>>>,
}

enum Message {
    Samples(Vec<f32>),
    Lost,
}

impl Recorder {
    /// Opens the microphone and starts recording. Returns once the stream is
    /// running, or with the error that prevented it.
    pub fn start(options: RecorderOptions) -> Result<Self, AudioError> {
        let (stop_tx, stop_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("yap-audio".into())
            .spawn(move || run(options, stop_rx, ready_tx))
            .map_err(|e| AudioError::Other(e.to_string()))?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                stop: stop_tx,
                thread: Some(thread),
            }),
            Ok(Err(error)) => {
                let _ = thread.join();
                Err(error)
            }
            Err(_) => Err(AudioError::Other("the audio thread stopped".into())),
        }
    }

    /// Stops recording and returns the audio as 16 kHz mono samples.
    pub fn stop(mut self) -> Vec<f32> {
        self.finish()
    }

    fn finish(&mut self) -> Vec<f32> {
        let _ = self.stop.send(());
        self.thread
            .take()
            .and_then(|thread| thread.join().ok())
            .unwrap_or_default()
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.finish();
        }
    }
}

fn run(
    options: RecorderOptions,
    stop: Receiver<()>,
    ready: Sender<Result<(), AudioError>>,
) -> Vec<f32> {
    let RecorderOptions {
        device_id,
        max_duration,
        keep_audio,
        mut on_level,
        on_end,
    } = options;

    let (tx, rx) = mpsc::channel::<Message>();
    let (stream, sample_rate) = match open_stream(&device_id, tx) {
        Ok(opened) => opened,
        Err(error) => {
            let _ = ready.send(Err(error));
            return Vec::new();
        }
    };
    if let Err(error) = stream.play() {
        let _ = ready.send(Err(error.into()));
        return Vec::new();
    }
    let _ = ready.send(Ok(()));

    let mut resampler = keep_audio.then(|| StreamResampler::new(sample_rate));
    let mut meter = LevelMeter::new(sample_rate);
    let started = Instant::now();
    let mut ended = None;

    let mut handle = |samples: &[f32], meter: &mut LevelMeter| {
        if let Some(resampler) = resampler.as_mut() {
            resampler.push(samples);
        }
        meter.push(samples, &mut on_level);
    };

    loop {
        match stop.try_recv() {
            Ok(()) | Err(mpsc::TryRecvError::Disconnected) => break,
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if started.elapsed() >= max_duration {
            ended = Some(EndReason::TimeLimit);
            break;
        }
        match rx.recv_timeout(Duration::from_millis(20)) {
            Ok(Message::Samples(samples)) => handle(&samples, &mut meter),
            Ok(Message::Lost) => {
                ended = Some(EndReason::DeviceLost);
                break;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                ended = Some(EndReason::DeviceLost);
                break;
            }
        }
    }

    drop(stream);
    // Keep whatever the callback delivered before the stream closed.
    while let Ok(Message::Samples(samples)) = rx.try_recv() {
        handle(&samples, &mut meter);
    }

    if let Some(reason) = ended {
        log::warn!("recording ended on its own: {reason:?}");
        on_end(reason);
    }
    resampler.map(StreamResampler::finish).unwrap_or_default()
}

fn open_stream(device_id: &str, tx: Sender<Message>) -> Result<(cpal::Stream, u32), AudioError> {
    let device = device::resolve(device_id)?;
    let supported = device.default_input_config()?;
    let format = supported.sample_format();
    let config = supported.config();
    let channels = usize::from(config.channels.max(1));
    let sample_rate = config.sample_rate;

    let error_tx = tx.clone();
    let on_error = move |err: cpal::Error| {
        log::warn!("microphone stream error: {err}");
        if matches!(
            err.kind(),
            cpal::ErrorKind::DeviceNotAvailable | cpal::ErrorKind::StreamInvalidated
        ) {
            let _ = error_tx.send(Message::Lost);
        }
    };

    macro_rules! build {
        ($t:ty) => {
            device.build_input_stream::<$t, _, _>(
                config,
                move |data: &[$t], _| {
                    let mut mono = Vec::with_capacity(data.len() / channels);
                    mix_to_mono(data, channels, &mut mono);
                    let _ = tx.send(Message::Samples(mono));
                },
                on_error,
                None,
            )
        };
    }

    let stream = match format {
        SampleFormat::F32 => build!(f32),
        SampleFormat::I16 => build!(i16),
        SampleFormat::U16 => build!(u16),
        SampleFormat::I32 => build!(i32),
        SampleFormat::U8 => build!(u8),
        SampleFormat::I8 => build!(i8),
        SampleFormat::F64 => build!(f64),
        other => return Err(AudioError::Unsupported(format!("{other} samples"))),
    }?;
    Ok((stream, sample_rate))
}

/// Averages interleaved channels into mono f32 samples, appended to `out`.
pub fn mix_to_mono<T>(interleaved: &[T], channels: usize, out: &mut Vec<f32>)
where
    T: Copy,
    f32: FromSample<T>,
{
    let channels = channels.max(1);
    for frame in interleaved.chunks_exact(channels) {
        let sum: f32 = frame.iter().map(|&s| f32::from_sample_(s)).sum();
        out.push(sum / channels as f32);
    }
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0, |max, s| max.max(s.abs()))
}

/// Groups incoming samples into ~33 ms windows and reports each one's level.
struct LevelMeter {
    window: usize,
    buffer: Vec<f32>,
}

impl LevelMeter {
    fn new(sample_rate: u32) -> Self {
        let window = (sample_rate as f64 * LEVEL_WINDOW.as_secs_f64()) as usize;
        Self {
            window: window.max(1),
            buffer: Vec::with_capacity(window),
        }
    }

    fn push(&mut self, samples: &[f32], on_level: &mut dyn FnMut(Level)) {
        for &sample in samples {
            self.buffer.push(sample);
            if self.buffer.len() == self.window {
                on_level(Level {
                    rms: rms(&self.buffer),
                    peak: peak(&self.buffer),
                });
                self.buffer.clear();
            }
        }
    }
}

/// Resamples a mono stream to 16 kHz chunk by chunk, so long recordings never
/// hold the full-rate audio in memory.
pub struct StreamResampler {
    inner: Option<Fft<f32>>,
    pending: Vec<f32>,
    output: Vec<f32>,
    /// Output frames still to drop: the resampler's startup delay.
    delay_left: usize,
    input_total: usize,
    ratio: f64,
}

impl StreamResampler {
    pub fn new(input_rate: u32) -> Self {
        let ratio = f64::from(TARGET_SAMPLE_RATE) / f64::from(input_rate);
        let inner = (input_rate != TARGET_SAMPLE_RATE).then(|| {
            Fft::<f32>::new(
                input_rate as usize,
                TARGET_SAMPLE_RATE as usize,
                RESAMPLER_CHUNK,
                1,
                FixedSync::Input,
            )
            .expect("valid resampler settings")
        });
        let delay_left = inner.as_ref().map_or(0, |r| r.output_delay());
        Self {
            inner,
            pending: Vec::new(),
            output: Vec::new(),
            delay_left,
            input_total: 0,
            ratio,
        }
    }

    pub fn push(&mut self, samples: &[f32]) {
        self.input_total += samples.len();
        let Some(resampler) = self.inner.as_mut() else {
            self.output.extend_from_slice(samples);
            return;
        };
        self.pending.extend_from_slice(samples);
        let mut offset = 0;
        loop {
            let needed = resampler.input_frames_next();
            if self.pending.len() - offset < needed {
                break;
            }
            let chunk = &self.pending[offset..offset + needed];
            Self::process(
                resampler,
                chunk,
                None,
                &mut self.output,
                &mut self.delay_left,
            );
            offset += needed;
        }
        self.pending.drain(..offset);
    }

    /// Flushes the remaining input and returns all output, exactly
    /// `input * 16000 / input_rate` samples long.
    pub fn finish(mut self) -> Vec<f32> {
        let Some(resampler) = self.inner.as_mut() else {
            return self.output;
        };
        let expected = (self.input_total as f64 * self.ratio).round() as usize;
        let needed = resampler.input_frames_next();
        let mut last = std::mem::take(&mut self.pending);
        let partial = last.len();
        last.resize(needed, 0.0);
        Self::process(
            resampler,
            &last,
            Some(partial),
            &mut self.output,
            &mut self.delay_left,
        );
        // Push silence through until the delayed tail has come out.
        let silence = vec![0.0; needed];
        let mut guard = 0;
        while self.output.len() < expected && guard < 64 {
            Self::process(
                resampler,
                &silence,
                Some(0),
                &mut self.output,
                &mut self.delay_left,
            );
            guard += 1;
        }
        self.output.truncate(expected);
        self.output
    }

    fn process(
        resampler: &mut Fft<f32>,
        input: &[f32],
        partial_len: Option<usize>,
        output: &mut Vec<f32>,
        delay_left: &mut usize,
    ) {
        let input_adapter =
            InterleavedSlice::new(input, 1, input.len()).expect("buffer matches its length");
        let mut buffer = vec![0.0; resampler.output_frames_next()];
        let frames = buffer.len();
        let mut output_adapter =
            InterleavedSlice::new_mut(&mut buffer, 1, frames).expect("buffer matches its length");
        let indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            partial_len,
            active_channels_mask: None,
        };
        let Ok((_, produced)) =
            resampler.process_into_buffer(&input_adapter, &mut output_adapter, Some(&indexing))
        else {
            return;
        };
        let skip = (*delay_left).min(produced);
        *delay_left -= skip;
        output.extend_from_slice(&buffer[skip..produced]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixes_stereo_to_mono() {
        let mut out = Vec::new();
        mix_to_mono(&[1.0f32, 0.0, 0.5, 0.5, -1.0, 1.0], 2, &mut out);
        assert_eq!(out, vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn mixes_integer_formats() {
        let mut out = Vec::new();
        mix_to_mono(&[i16::MAX, i16::MAX, 0, 0], 2, &mut out);
        assert!((out[0] - 1.0).abs() < 0.001);
        assert_eq!(out[1], 0.0);

        let mut out = Vec::new();
        mix_to_mono(&[u16::MAX / 2 + 1], 1, &mut out);
        assert!(out[0].abs() < 0.001, "u16 midpoint is silence");
    }

    #[test]
    fn rms_and_peak() {
        assert_eq!(rms(&[]), 0.0);
        assert!((rms(&[1.0, -1.0, 1.0, -1.0]) - 1.0).abs() < 1e-6);
        assert!((rms(&[0.5; 100]) - 0.5).abs() < 1e-6);
        assert_eq!(peak(&[0.1, -0.7, 0.3]), 0.7);
    }

    fn sine(rate: u32, seconds: f32, hz: f32) -> Vec<f32> {
        (0..(rate as f32 * seconds) as usize)
            .map(|i| (i as f32 / rate as f32 * hz * std::f32::consts::TAU).sin() * 0.5)
            .collect()
    }

    #[test]
    fn resamples_to_the_right_length() {
        for rate in [48_000, 44_100, 22_050, 16_000, 8_000] {
            let input = sine(rate, 1.3, 440.0);
            let mut resampler = StreamResampler::new(rate);
            // Feed in uneven pieces, as an audio callback would.
            for piece in input.chunks(333) {
                resampler.push(piece);
            }
            let output = resampler.finish();
            let expected = (input.len() as f64 * 16_000.0 / f64::from(rate)).round() as usize;
            assert_eq!(output.len(), expected, "{rate} Hz");
        }
    }

    #[test]
    fn resampling_keeps_the_signal() {
        let input = sine(48_000, 1.0, 440.0);
        let mut resampler = StreamResampler::new(48_000);
        resampler.push(&input);
        let output = resampler.finish();
        // A 0.5 amplitude sine has an RMS of about 0.354, before and after.
        let middle = &output[2_000..14_000];
        assert!((rms(middle) - 0.354).abs() < 0.02, "rms {}", rms(middle));
    }

    #[test]
    fn level_meter_reports_each_window() {
        let mut meter = LevelMeter::new(48_000);
        let mut levels = Vec::new();
        meter.push(&vec![0.25; 48_000], &mut |level| levels.push(level));
        assert!((29..=31).contains(&levels.len()), "{}", levels.len());
        assert!((levels[0].rms - 0.25).abs() < 1e-6);
    }
}

#[cfg(test)]
mod hardware_tests {
    use super::*;

    /// Run with `cargo test -- --ignored real_microphone --nocapture`.
    #[test]
    #[ignore = "needs a real microphone"]
    fn real_microphone_records_16khz() {
        for device in device::list().unwrap() {
            println!("{:>5} {} ({})", device.is_default, device.name, device.id);
        }
        let levels = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = levels.clone();
        let recorder = Recorder::start(RecorderOptions {
            device_id: device::DEFAULT_DEVICE.into(),
            max_duration: Duration::from_secs(5),
            keep_audio: true,
            on_level: Box::new(move |_| {
                counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }),
            on_end: Box::new(|_| {}),
        })
        .unwrap();
        std::thread::sleep(Duration::from_secs(1));
        let samples = recorder.stop();
        let levels = levels.load(std::sync::atomic::Ordering::Relaxed);
        println!(
            "{} samples, {levels} level updates, rms {}",
            samples.len(),
            rms(&samples)
        );
        assert!((15_000..=17_500).contains(&samples.len()));
        assert!((25..=35).contains(&levels));
    }
}
