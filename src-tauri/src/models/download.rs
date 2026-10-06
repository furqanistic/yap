//! Streaming, resumable, checksum-verified downloads.
//!
//! Data goes to `<file>.part`. If a `.part` already exists, the download
//! resumes with an HTTP `Range` request. When all bytes are in, the file is
//! hashed and only then renamed to its final name, so a file with the final
//! name is always complete.

use std::fmt;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::{header, StatusCode, Url};
use serde::ser::SerializeStruct;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use super::catalog::ModelInfo;

/// Progress callbacks fire at most this often (about 4 times a second).
const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);
/// Extra free space required on top of the download itself.
const DISK_MARGIN_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub enum ModelError {
    Offline,
    /// `needed` is the shortfall in bytes, when known.
    DiskFull {
        needed: Option<u64>,
    },
    HttpStatus(u16),
    ChecksumMismatch,
    Cancelled,
    Io(String),
    /// Another download is already running (one at a time in v1).
    Busy,
    UnknownModel(String),
    /// A URL or redirect pointed outside the host allowlist.
    NotAllowed(String),
}

impl ModelError {
    /// Stable identifier the frontend can switch on.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Offline => "offline",
            Self::DiskFull { .. } => "diskFull",
            Self::HttpStatus(_) => "httpStatus",
            Self::ChecksumMismatch => "checksumMismatch",
            Self::Cancelled => "cancelled",
            Self::Io(_) => "io",
            Self::Busy => "busy",
            Self::UnknownModel(_) => "unknownModel",
            Self::NotAllowed(_) => "notAllowed",
        }
    }
}

/// Plain-language messages, shown as-is in the UI.
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Offline => write!(
                f,
                "Can't reach Hugging Face. Check your internet connection and try again."
            ),
            Self::DiskFull {
                needed: Some(bytes),
            } => write!(
                f,
                "Not enough disk space. Free up {} and try again.",
                format_bytes(*bytes)
            ),
            Self::DiskFull { needed: None } => {
                write!(
                    f,
                    "Not enough disk space. Free up some space and try again."
                )
            }
            Self::HttpStatus(status) => write!(
                f,
                "The download server returned an error ({status}). Try again later."
            ),
            Self::ChecksumMismatch => write!(f, "The downloaded file was damaged. Try again."),
            Self::Cancelled => write!(f, "Download cancelled."),
            Self::Io(message) => write!(f, "Couldn't save the model: {message}"),
            Self::Busy => write!(f, "Another model is already downloading."),
            Self::UnknownModel(id) => write!(f, "Unknown model \"{id}\"."),
            Self::NotAllowed(_) => write!(f, "Blocked a download from an untrusted address."),
        }
    }
}

impl std::error::Error for ModelError {}

/// Serialized as `{ kind, message }` for the frontend.
impl Serialize for ModelError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ModelError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl From<io::Error> for ModelError {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::StorageFull {
            Self::DiskFull { needed: None }
        } else {
            Self::Io(err.to_string())
        }
    }
}

fn network_error(err: reqwest::Error) -> ModelError {
    if err.is_redirect() {
        ModelError::NotAllowed(err.url().map(Url::to_string).unwrap_or_default())
    } else if let Some(status) = err.status() {
        ModelError::HttpStatus(status.as_u16())
    } else {
        ModelError::Offline
    }
}

fn format_bytes(bytes: u64) -> String {
    const MB: f64 = 1024.0 * 1024.0;
    let mb = bytes as f64 / MB;
    if mb >= 1024.0 {
        format!("{:.1} GB", mb / 1024.0)
    } else {
        format!("{} MB", mb.ceil() as u64)
    }
}

/// Which URLs downloads may use, including every redirect hop.
#[derive(Debug, Clone)]
pub struct UrlPolicy {
    allow_local_http: bool,
}

impl UrlPolicy {
    /// HTTPS to Hugging Face and its CDNs only.
    pub fn huggingface() -> Self {
        Self {
            allow_local_http: false,
        }
    }

    /// Also allows plain HTTP to 127.0.0.1, for tests with a local server.
    #[cfg(test)]
    pub fn local_test() -> Self {
        Self {
            allow_local_http: true,
        }
    }

    pub fn allows(&self, url: &Url) -> bool {
        let Some(host) = url.host_str() else {
            return false;
        };
        if self.allow_local_http && url.scheme() == "http" && host == "127.0.0.1" {
            return true;
        }
        let trusted = ["huggingface.co", "hf.co"]
            .iter()
            .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")));
        url.scheme() == "https" && trusted
    }

    /// An HTTP client that refuses redirects outside this policy.
    pub fn client(&self) -> reqwest::Client {
        let policy = self.clone();
        reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 10 {
                    attempt.error("too many redirects")
                } else if policy.allows(attempt.url()) {
                    attempt.follow()
                } else {
                    attempt.error("redirect outside the allowed hosts")
                }
            }))
            .connect_timeout(Duration::from_secs(15))
            // A stalled connection errors out instead of hanging forever.
            .read_timeout(Duration::from_secs(30))
            .user_agent(concat!("Yap/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("HTTP client configuration is valid")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub downloaded: u64,
    pub total: u64,
    pub bytes_per_sec: u64,
}

pub fn final_path(dir: &Path, model: &ModelInfo) -> PathBuf {
    dir.join(model.file_name)
}

pub fn part_path(dir: &Path, model: &ModelInfo) -> PathBuf {
    dir.join(format!("{}.part", model.file_name))
}

/// A model is ready only if its file exists with exactly the expected size.
pub fn is_ready(dir: &Path, model: &ModelInfo) -> bool {
    std::fs::metadata(final_path(dir, model))
        .map(|meta| meta.is_file() && meta.len() == model.size_bytes)
        .unwrap_or(false)
}

/// Bytes already saved by an earlier, unfinished download.
pub fn partial_bytes(dir: &Path, model: &ModelInfo) -> u64 {
    std::fs::metadata(part_path(dir, model))
        .map(|meta| meta.len())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpace {
    pub available_bytes: u64,
    /// What's still to download, plus a safety margin.
    pub required_bytes: u64,
}

/// Free space in `dir` compared with what downloading `bytes` needs.
pub fn disk_space(dir: &Path, bytes: u64) -> Result<DiskSpace, ModelError> {
    Ok(DiskSpace {
        available_bytes: fs4::available_space(dir)?,
        required_bytes: bytes + DISK_MARGIN_BYTES,
    })
}

/// Fails with `DiskFull` unless `dir` has room for `bytes` plus a margin.
pub fn check_disk_space(dir: &Path, bytes: u64) -> Result<(), ModelError> {
    let space = disk_space(dir, bytes)?;
    if space.available_bytes < space.required_bytes {
        return Err(ModelError::DiskFull {
            needed: Some(space.required_bytes - space.available_bytes),
        });
    }
    Ok(())
}

/// Smoothed transfer speed, sampled every `PROGRESS_INTERVAL`.
struct SpeedMeter {
    last_at: Instant,
    last_bytes: u64,
    speed: f64,
}

impl SpeedMeter {
    fn new(bytes: u64) -> Self {
        Self {
            last_at: Instant::now(),
            last_bytes: bytes,
            speed: 0.0,
        }
    }

    /// Returns the new speed when it's time to report progress again.
    fn tick(&mut self, bytes: u64) -> Option<u64> {
        let elapsed = self.last_at.elapsed();
        if elapsed < PROGRESS_INTERVAL {
            return None;
        }
        let current = (bytes - self.last_bytes) as f64 / elapsed.as_secs_f64();
        self.speed = if self.speed == 0.0 {
            current
        } else {
            0.3 * current + 0.7 * self.speed
        };
        self.last_at = Instant::now();
        self.last_bytes = bytes;
        Some(self.speed as u64)
    }
}

/// Downloads `model` into `dir`, resuming any `.part` file, then verifies it.
///
/// Cancelling keeps the `.part` file so a later call resumes. A checksum
/// mismatch deletes it, since resuming would only repeat the damage.
pub async fn download(
    client: &reqwest::Client,
    policy: &UrlPolicy,
    model: &ModelInfo,
    dir: &Path,
    cancel: &CancellationToken,
    mut on_progress: impl FnMut(Progress),
    on_verifying: impl FnOnce(),
) -> Result<(), ModelError> {
    let url: Url = model
        .url
        .parse()
        .map_err(|_| ModelError::NotAllowed(model.url.into()))?;
    if !policy.allows(&url) {
        return Err(ModelError::NotAllowed(model.url.into()));
    }

    fs::create_dir_all(dir).await?;
    let part = part_path(dir, model);
    let total = model.size_bytes;
    let mut offset = partial_bytes(dir, model);
    if offset > total {
        fs::remove_file(&part).await?;
        offset = 0;
    }

    if offset < total {
        let mut request = client.get(url);
        if offset > 0 {
            request = request.header(header::RANGE, format!("bytes={offset}-"));
        }
        let response = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(ModelError::Cancelled),
            response = request.send() => response.map_err(network_error)?,
        };

        let append = match response.status() {
            StatusCode::PARTIAL_CONTENT if offset > 0 => {
                let expected = format!("bytes {offset}-");
                let matches = response
                    .headers()
                    .get(header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok())
                    .is_some_and(|value| value.starts_with(&expected));
                if !matches {
                    fs::remove_file(&part).await.ok();
                    return Err(ModelError::HttpStatus(206));
                }
                true
            }
            // The server ignored the range: start over.
            StatusCode::OK => {
                offset = 0;
                false
            }
            status => {
                if status == StatusCode::RANGE_NOT_SATISFIABLE {
                    fs::remove_file(&part).await.ok();
                }
                return Err(ModelError::HttpStatus(status.as_u16()));
            }
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(&part)
            .await?;

        let mut downloaded = offset;
        let mut meter = SpeedMeter::new(downloaded);
        let mut stream = response.bytes_stream();
        on_progress(Progress {
            downloaded,
            total,
            bytes_per_sec: 0,
        });

        loop {
            let chunk = tokio::select! {
                biased;
                _ = cancel.cancelled() => {
                    file.flush().await?;
                    return Err(ModelError::Cancelled);
                }
                chunk = stream.next() => chunk,
            };
            let Some(chunk) = chunk else { break };
            let chunk = chunk.map_err(network_error)?;

            downloaded += chunk.len() as u64;
            if downloaded > total {
                drop(file);
                fs::remove_file(&part).await.ok();
                return Err(ModelError::ChecksumMismatch);
            }
            file.write_all(&chunk).await?;

            if let Some(bytes_per_sec) = meter.tick(downloaded) {
                on_progress(Progress {
                    downloaded,
                    total,
                    bytes_per_sec,
                });
            }
        }

        file.flush().await?;
        file.sync_all().await?;
        drop(file);

        // The connection closed early. Keep what we have for a resume.
        if downloaded < total {
            return Err(ModelError::Offline);
        }
        on_progress(Progress {
            downloaded,
            total,
            bytes_per_sec: meter.speed as u64,
        });
    }

    on_verifying();
    let hash = sha256_file(part.clone()).await?;
    if !hash.eq_ignore_ascii_case(model.sha256) {
        fs::remove_file(&part).await.ok();
        return Err(ModelError::ChecksumMismatch);
    }
    fs::rename(&part, final_path(dir, model)).await?;
    Ok(())
}

/// Hashes a file on a blocking thread, so large models don't stall async work.
async fn sha256_file(path: PathBuf) -> Result<String, ModelError> {
    let hash = tokio::task::spawn_blocking(move || -> io::Result<String> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0; 1024 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect())
    })
    .await
    .map_err(|err| ModelError::Io(err.to_string()))??;
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::catalog::Languages;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    /// A one-file HTTP server on 127.0.0.1 that understands `Range: bytes=N-`.
    /// Records the Range header of every request.
    struct TestServer {
        url: String,
        ranges: Arc<Mutex<Vec<Option<String>>>>,
    }

    impl TestServer {
        fn start(body: Vec<u8>, honor_range: bool) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let url = format!("http://{}/model.bin", listener.local_addr().unwrap());
            let ranges = Arc::new(Mutex::new(Vec::new()));
            let seen = ranges.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { break };
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut range = None;
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                            break;
                        }
                        if let Some(value) = line.to_ascii_lowercase().strip_prefix("range:") {
                            range = Some(value.trim().to_string());
                        }
                    }
                    seen.lock().unwrap().push(range.clone());

                    let start: Option<usize> = range
                        .filter(|_| honor_range)
                        .and_then(|r| r.strip_prefix("bytes=")?.strip_suffix('-')?.parse().ok());
                    let (status, slice, extra) = match start {
                        Some(start) => (
                            "206 Partial Content",
                            &body[start..],
                            format!(
                                "Content-Range: bytes {start}-{}/{}\r\n",
                                body.len() - 1,
                                body.len()
                            ),
                        ),
                        None => ("200 OK", &body[..], String::new()),
                    };
                    let head = format!(
                        "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n",
                        slice.len()
                    );
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(slice);
                }
            });
            Self { url, ranges }
        }

        fn model(&self, body: &[u8], sha256: Option<&str>) -> ModelInfo {
            let hash = sha256.map(str::to_string).unwrap_or_else(|| {
                Sha256::digest(body)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect()
            });
            ModelInfo {
                id: "test",
                label: "Test",
                description: "",
                languages: Languages::Multilingual,
                recommended: false,
                size_bytes: body.len() as u64,
                url: Box::leak(self.url.clone().into_boxed_str()),
                sha256: Box::leak(hash.into_boxed_str()),
                file_name: "model.bin",
            }
        }
    }

    fn fixture() -> Vec<u8> {
        (0..200_000u32).map(|i| (i % 251) as u8).collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("yap-models-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    async fn run(model: &ModelInfo, dir: &Path) -> Result<(), ModelError> {
        let policy = UrlPolicy::local_test();
        download(
            &policy.client(),
            &policy,
            model,
            dir,
            &CancellationToken::new(),
            |_| {},
            || {},
        )
        .await
    }

    #[tokio::test]
    async fn downloads_and_verifies() {
        let body = fixture();
        let server = TestServer::start(body.clone(), true);
        let model = server.model(&body, None);
        let dir = temp_dir("ok");

        assert!(!is_ready(&dir, &model));
        run(&model, &dir).await.unwrap();
        assert!(is_ready(&dir, &model));
        assert!(!part_path(&dir, &model).exists());
        assert_eq!(std::fs::read(final_path(&dir, &model)).unwrap(), body);
    }

    #[tokio::test]
    async fn resumes_from_a_partial_file() {
        let body = fixture();
        let server = TestServer::start(body.clone(), true);
        let model = server.model(&body, None);
        let dir = temp_dir("resume");
        std::fs::write(part_path(&dir, &model), &body[..70_000]).unwrap();

        run(&model, &dir).await.unwrap();
        assert_eq!(
            server.ranges.lock().unwrap().as_slice(),
            &[Some("bytes=70000-".to_string())]
        );
        assert_eq!(std::fs::read(final_path(&dir, &model)).unwrap(), body);
    }

    #[tokio::test]
    async fn restarts_when_the_server_ignores_range() {
        let body = fixture();
        let server = TestServer::start(body.clone(), false);
        let model = server.model(&body, None);
        let dir = temp_dir("norange");
        std::fs::write(part_path(&dir, &model), b"garbage that must be discarded").unwrap();

        run(&model, &dir).await.unwrap();
        assert_eq!(std::fs::read(final_path(&dir, &model)).unwrap(), body);
    }

    #[tokio::test]
    async fn rejects_a_checksum_mismatch() {
        let body = fixture();
        let server = TestServer::start(body.clone(), true);
        let wrong = "0".repeat(64);
        let model = server.model(&body, Some(&wrong));
        let dir = temp_dir("mismatch");

        assert_eq!(run(&model, &dir).await, Err(ModelError::ChecksumMismatch));
        assert!(!part_path(&dir, &model).exists());
        assert!(!final_path(&dir, &model).exists());
    }

    #[tokio::test]
    async fn ready_requires_the_exact_size() {
        let body = fixture();
        let server = TestServer::start(body.clone(), true);
        let model = server.model(&body, None);
        let dir = temp_dir("size");
        std::fs::write(final_path(&dir, &model), &body[..10]).unwrap();
        assert!(!is_ready(&dir, &model));
        std::fs::write(final_path(&dir, &model), &body).unwrap();
        assert!(is_ready(&dir, &model));
    }

    #[tokio::test]
    async fn cancelling_keeps_the_partial_file() {
        let body = fixture();
        let server = TestServer::start(body.clone(), true);
        let model = server.model(&body, None);
        let dir = temp_dir("cancel");
        let policy = UrlPolicy::local_test();
        let cancel = CancellationToken::new();
        cancel.cancel();
        let result = download(
            &policy.client(),
            &policy,
            &model,
            &dir,
            &cancel,
            |_| {},
            || {},
        )
        .await;
        assert_eq!(result, Err(ModelError::Cancelled));
        assert!(!final_path(&dir, &model).exists());
    }

    #[tokio::test]
    async fn offline_gives_a_clear_error() {
        // Bind then drop a listener so the port is closed.
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let model = ModelInfo {
            url: Box::leak(format!("http://127.0.0.1:{port}/m.bin").into_boxed_str()),
            ..TestServer {
                url: String::new(),
                ranges: Default::default(),
            }
            .model(b"x", None)
        };
        let result = run(&model, &temp_dir("offline")).await;
        assert_eq!(result, Err(ModelError::Offline));
        assert!(result.unwrap_err().to_string().contains("internet"));
    }

    #[test]
    fn url_policy_only_allows_hugging_face_over_https() {
        let policy = UrlPolicy::huggingface();
        let allows = |url: &str| policy.allows(&url.parse().unwrap());
        assert!(allows("https://huggingface.co/x"));
        assert!(allows("https://cdn-lfs.huggingface.co/x"));
        assert!(allows("https://us.aws.cdn.hf.co/x"));
        assert!(!allows("http://huggingface.co/x"));
        assert!(!allows("https://evilhuggingface.co/x"));
        assert!(!allows("https://huggingface.co.evil.com/x"));
        assert!(!allows("http://127.0.0.1/x"));
    }

    #[test]
    fn errors_serialize_with_kind_and_message() {
        let value = serde_json::to_value(ModelError::DiskFull {
            needed: Some(3 * 1024 * 1024 * 1024),
        })
        .unwrap();
        assert_eq!(value["kind"], "diskFull");
        assert_eq!(
            value["message"],
            "Not enough disk space. Free up 3.0 GB and try again."
        );
    }

    /// The real thing: HTTPS, CDN redirects, cancel, resume and checksum.
    /// Run with `cargo test -- --ignored real_tiny`.
    #[tokio::test]
    #[ignore = "downloads 78 MB from Hugging Face"]
    async fn real_tiny_download_cancels_and_resumes() {
        let model = crate::models::catalog::find("whisper-tiny").unwrap();
        let dir = temp_dir("real");
        let policy = UrlPolicy::huggingface();
        let client = policy.client();

        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let first = download(
            &client,
            &policy,
            model,
            &dir,
            &cancel,
            |p| {
                if p.downloaded > 20_000_000 {
                    stop.cancel();
                }
            },
            || {},
        )
        .await;
        assert_eq!(first, Err(ModelError::Cancelled));
        let partial = partial_bytes(&dir, model);
        assert!(partial > 20_000_000 && partial < model.size_bytes);

        let mut resumed_from = None;
        download(
            &client,
            &policy,
            model,
            &dir,
            &CancellationToken::new(),
            |p| {
                resumed_from.get_or_insert(p.downloaded);
            },
            || {},
        )
        .await
        .unwrap();
        assert_eq!(resumed_from, Some(partial));
        assert!(is_ready(&dir, model));
        let _ = std::fs::remove_dir_all(dir);
    }
}
