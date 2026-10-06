//! Speech model catalog, downloads and on-disk state.
//!
//! Models are never bundled and never download in the background: the user
//! starts each download. One download runs at a time.

pub mod catalog;
pub mod download;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

use catalog::ModelInfo;
use download::{DiskSpace, ModelError, Progress, UrlPolicy};

pub const PROGRESS_EVENT: &str = "models://progress";
pub const STATE_EVENT: &str = "models://state";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ModelState {
    /// `partial_bytes` > 0 means a cancelled or interrupted download can resume.
    NotDownloaded {
        partial_bytes: u64,
    },
    Downloading {
        downloaded: u64,
        total: u64,
    },
    Verifying,
    Ready,
    Failed {
        error: ModelError,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
    #[serde(flatten)]
    pub info: &'static ModelInfo,
    pub state: ModelState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StateEvent {
    id: &'static str,
    state: ModelState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    id: &'static str,
    #[serde(flatten)]
    progress: Progress,
}

enum Phase {
    Downloading(Progress),
    Verifying,
}

struct Active {
    id: &'static str,
    cancel: CancellationToken,
    phase: Phase,
}

#[derive(Default)]
struct Inner {
    active: Option<Active>,
    /// The last error per model, cleared when a new download starts.
    failures: HashMap<&'static str, ModelError>,
}

/// Held in Tauri managed state.
pub struct ModelManager {
    dir: PathBuf,
    policy: UrlPolicy,
    client: reqwest::Client,
    inner: Mutex<Inner>,
}

impl ModelManager {
    pub fn new(dir: PathBuf) -> Self {
        let policy = UrlPolicy::huggingface();
        Self {
            dir,
            client: policy.client(),
            policy,
            inner: Mutex::default(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Free space compared with what downloading model `id` still needs.
    pub fn disk_space(&self, id: &str) -> Result<DiskSpace, ModelError> {
        let model = catalog::find(id).ok_or_else(|| ModelError::UnknownModel(id.into()))?;
        std::fs::create_dir_all(&self.dir)?;
        let partial = download::partial_bytes(&self.dir, model).min(model.size_bytes);
        download::disk_space(&self.dir, model.size_bytes - partial)
    }

    pub fn list(&self) -> Vec<ModelEntry> {
        let inner = self.lock();
        catalog::CATALOG
            .iter()
            .map(|info| ModelEntry {
                info,
                state: self.state_locked(&inner, info),
            })
            .collect()
    }

    fn state_locked(&self, inner: &Inner, model: &ModelInfo) -> ModelState {
        if let Some(active) = inner.active.as_ref().filter(|a| a.id == model.id) {
            return match active.phase {
                Phase::Downloading(p) => ModelState::Downloading {
                    downloaded: p.downloaded,
                    total: p.total,
                },
                Phase::Verifying => ModelState::Verifying,
            };
        }
        if download::is_ready(&self.dir, model) {
            return ModelState::Ready;
        }
        if let Some(error) = inner.failures.get(model.id) {
            return ModelState::Failed {
                error: error.clone(),
            };
        }
        ModelState::NotDownloaded {
            partial_bytes: download::partial_bytes(&self.dir, model),
        }
    }

    /// Starts a download in the background. Progress and the outcome arrive
    /// as `models://progress` and `models://state` events.
    pub fn start_download(&self, app: &AppHandle, id: &str) -> Result<(), ModelError> {
        let model = catalog::find(id).ok_or_else(|| ModelError::UnknownModel(id.into()))?;
        if download::is_ready(&self.dir, model) {
            return Ok(());
        }

        let cancel = {
            let mut inner = self.lock();
            if inner.active.is_some() {
                return Err(ModelError::Busy);
            }
            std::fs::create_dir_all(&self.dir)?;
            let partial = download::partial_bytes(&self.dir, model).min(model.size_bytes);
            download::check_disk_space(&self.dir, model.size_bytes - partial)?;

            let cancel = CancellationToken::new();
            inner.failures.remove(model.id);
            inner.active = Some(Active {
                id: model.id,
                cancel: cancel.clone(),
                phase: Phase::Downloading(Progress {
                    downloaded: partial,
                    total: model.size_bytes,
                    bytes_per_sec: 0,
                }),
            });
            cancel
        };
        self.emit_state(app, model);

        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let manager = app.state::<ModelManager>();
            let result = download::download(
                &manager.client,
                &manager.policy,
                model,
                &manager.dir,
                &cancel,
                |progress| manager.on_progress(&app, model, progress),
                || manager.on_verifying(&app, model),
            )
            .await;
            manager.finish(&app, model, result);
        });
        Ok(())
    }

    /// Stops the download. The `.part` file stays so it can resume later.
    pub fn cancel(&self, id: &str) {
        if let Some(active) = self.lock().active.as_ref().filter(|a| a.id == id) {
            active.cancel.cancel();
        }
    }

    /// Deletes the model file and any partial download.
    // Step 11 adds: refuse while a dictation is using this model.
    pub fn delete(&self, app: &AppHandle, id: &str) -> Result<(), ModelError> {
        let model = catalog::find(id).ok_or_else(|| ModelError::UnknownModel(id.into()))?;
        {
            let mut inner = self.lock();
            if inner.active.as_ref().is_some_and(|a| a.id == model.id) {
                return Err(ModelError::Busy);
            }
            for path in [
                download::final_path(&self.dir, model),
                download::part_path(&self.dir, model),
            ] {
                match std::fs::remove_file(path) {
                    Err(err) if err.kind() != std::io::ErrorKind::NotFound => {
                        return Err(err.into())
                    }
                    _ => {}
                }
            }
            inner.failures.remove(model.id);
        }
        self.emit_state(app, model);
        Ok(())
    }

    fn on_progress(&self, app: &AppHandle, model: &'static ModelInfo, progress: Progress) {
        if let Some(active) = self.lock().active.as_mut() {
            active.phase = Phase::Downloading(progress);
        }
        let _ = app.emit(
            PROGRESS_EVENT,
            ProgressEvent {
                id: model.id,
                progress,
            },
        );
    }

    fn on_verifying(&self, app: &AppHandle, model: &'static ModelInfo) {
        if let Some(active) = self.lock().active.as_mut() {
            active.phase = Phase::Verifying;
        }
        self.emit_state(app, model);
    }

    fn finish(&self, app: &AppHandle, model: &'static ModelInfo, result: Result<(), ModelError>) {
        {
            let mut inner = self.lock();
            inner.active = None;
            match result {
                Ok(()) => log::info!("downloaded model {}", model.id),
                Err(ModelError::Cancelled) => log::info!("cancelled download of {}", model.id),
                Err(error) => {
                    log::warn!("download of {} failed: {error:?}", model.id);
                    inner.failures.insert(model.id, error);
                }
            }
        }
        self.emit_state(app, model);
    }

    fn emit_state(&self, app: &AppHandle, model: &'static ModelInfo) {
        let state = self.state_locked(&self.lock(), model);
        let _ = app.emit(
            STATE_EVENT,
            StateEvent {
                id: model.id,
                state,
            },
        );
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}
