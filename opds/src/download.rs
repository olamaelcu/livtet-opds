use std::{collections::HashMap, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use camino_tempfile::NamedUtf8TempFile;
use fs_err as fs;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tokio::{io::AsyncWriteExt, sync::RwLock};
use tracing::warn;

use crate::auth::OpdsAuth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DownloadStatus {
    Pending,
    InProgress {
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    Complete(String),
    Failed(String),
    Cancelled,
}

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("download not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: String,
    pub url: String,
    pub dest_path: Utf8PathBuf,
    pub status: DownloadStatus,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub auth: Option<OpdsAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStore {
    pub version: u32,
    pub downloads: Vec<DownloadRecord>,
}

#[derive(Debug)]
struct DownloadState {
    status: DownloadStatus,
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
    url: String,
    dest_path: Utf8PathBuf,
    auth: Option<OpdsAuth>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

pub struct DownloadManager {
    http: reqwest::Client,
    auth: OpdsAuth,
    downloads: Arc<RwLock<HashMap<String, DownloadState>>>,
    store_path: Utf8PathBuf,
}

impl DownloadManager {
    pub fn new(store_path: Utf8PathBuf, auth: OpdsAuth) -> Self {
        Self::with_http(store_path, auth, reqwest::Client::new())
    }

    /// Build a manager with an externally supplied `reqwest::Client`.
    pub fn with_http(store_path: Utf8PathBuf, auth: OpdsAuth, http: reqwest::Client) -> Self {
        Self {
            http,
            auth,
            downloads: Arc::new(RwLock::new(HashMap::new())),
            store_path,
        }
    }

    pub async fn load(&self) -> Result<usize, DownloadError> {
        if !self.store_path.exists() {
            return Ok(0);
        }
        let data = tokio::fs::read_to_string(&self.store_path).await?;
        let store: DownloadStore = serde_json::from_str(&data)?;
        let mut map = self.downloads.write().await;
        let count = store.downloads.len();
        for mut rec in store.downloads {
            if matches!(
                rec.status,
                DownloadStatus::InProgress { .. } | DownloadStatus::Pending
            ) {
                rec.status = DownloadStatus::Failed("App restarted during download".into());
            }
            map.insert(
                rec.id.clone(),
                DownloadState {
                    status: rec.status,
                    cancel_tx: None,
                    url: rec.url,
                    dest_path: rec.dest_path,
                    auth: rec.auth,
                    created_at: rec.created_at,
                    updated_at: rec.updated_at,
                },
            );
        }
        Ok(count)
    }

    async fn persist(&self) -> Result<(), DownloadError> {
        persist_inner(&self.downloads, &self.store_path).await
    }

    pub async fn start(&self, url: String, dest_path: Utf8PathBuf) -> Result<String, String> {
        let download_id = format!(
            "dl_{}",
            (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
        );
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
        let now = OffsetDateTime::now_utc();

        {
            let mut downloads = self.downloads.write().await;
            downloads.insert(
                download_id.clone(),
                DownloadState {
                    status: DownloadStatus::Pending,
                    cancel_tx: Some(cancel_tx),
                    url: url.clone(),
                    dest_path: dest_path.clone(),
                    auth: Some(self.auth.clone()),
                    created_at: now,
                    updated_at: now,
                },
            );
        }

        if let Err(e) = self.persist().await {
            warn!(error = %e, "failed to persist initial download record");
        }

        let http = self.http.clone();
        let auth = self.auth.clone();
        let downloads = self.downloads.clone();
        let store_path = self.store_path.clone();
        let id = download_id.clone();

        tokio::spawn(async move {
            download_task(
                url,
                dest_path,
                &mut cancel_rx,
                DownloadTaskCtx {
                    http,
                    auth,
                    download_id: id,
                    downloads,
                    store_path,
                },
            )
            .await;
        });

        Ok(download_id)
    }

    pub async fn status(&self, download_id: &str) -> Result<DownloadStatus, String> {
        let downloads = self.downloads.read().await;
        downloads
            .get(download_id)
            .map(|s| s.status.clone())
            .ok_or_else(|| format!("download not found: {}", download_id))
    }

    pub async fn cancel(&self, download_id: &str) -> Result<(), String> {
        let mut downloads = self.downloads.write().await;
        if let Some(state) = downloads.get_mut(download_id) {
            if let Some(tx) = state.cancel_tx.take() {
                let _ = tx.send(());
            }
            Ok(())
        } else {
            Err(format!("download not found: {}", download_id))
        }
    }

    pub async fn cleanup(&self, download_id: &str) -> Result<(), String> {
        let existed = {
            let mut downloads = self.downloads.write().await;
            downloads.remove(download_id).is_some()
        };
        if !existed {
            return Err(format!("download not found: {}", download_id));
        }
        if let Err(e) = self.persist().await {
            warn!(error = %e, "failed to persist after cleanup");
        }
        Ok(())
    }

    pub async fn cleanup_old(&self, age: time::Duration) -> Result<usize, DownloadError> {
        let cutoff = OffsetDateTime::now_utc() - age;
        let removed = {
            let mut downloads = self.downloads.write().await;
            let before = downloads.len();
            downloads.retain(|_id, state| state.created_at >= cutoff);
            before - downloads.len()
        };
        self.persist().await?;
        Ok(removed)
    }
}

struct DownloadTaskCtx {
    http: reqwest::Client,
    auth: OpdsAuth,
    download_id: String,
    downloads: Arc<RwLock<HashMap<String, DownloadState>>>,
    store_path: Utf8PathBuf,
}

async fn download_task(
    url: String,
    dest_path: Utf8PathBuf,
    cancel_rx: &mut tokio::sync::oneshot::Receiver<()>,
    ctx: DownloadTaskCtx,
) {
    let builder = ctx.http.get(&url);
    // Race the initial request against cancellation: a cancel issued while the
    // server is still sending headers must win, otherwise the download runs to
    // completion and reports `Complete` despite being cancelled.
    let response = tokio::select! {
        biased;
        _ = &mut *cancel_rx => {
            set_status_and_persist(
                &ctx.downloads,
                &ctx.store_path,
                &ctx.download_id,
                DownloadStatus::Cancelled,
            )
            .await;
            let _ = fs::tokio::remove_file(&dest_path).await;
            return;
        }
        response = ctx.auth.apply(builder).send() => match response {
            Ok(r) => r,
            Err(e) => {
                set_status_and_persist(
                    &ctx.downloads,
                    &ctx.store_path,
                    &ctx.download_id,
                    DownloadStatus::Failed(e.to_string()),
                )
                .await;
                return;
            }
        },
    };

    let total_bytes = response.content_length().unwrap_or(0);
    let mut bytes_downloaded = 0u64;

    let mut file = match fs::tokio::File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            set_status_and_persist(
                &ctx.downloads,
                &ctx.store_path,
                &ctx.download_id,
                DownloadStatus::Failed(e.to_string()),
            )
            .await;
            return;
        }
    };

    let mut stream = response.bytes_stream();

    loop {
        tokio::select! {
            biased;
            _ = &mut *cancel_rx => {
                set_status_and_persist(
                    &ctx.downloads,
                    &ctx.store_path,
                    &ctx.download_id,
                    DownloadStatus::Cancelled,
                )
                .await;
                let _ = fs::tokio::remove_file(&dest_path).await;
                return;
            }
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        if file.write_all(&bytes).await.is_err() {
                            set_status_and_persist(
                                &ctx.downloads,
                                &ctx.store_path,
                                &ctx.download_id,
                                DownloadStatus::Failed("write error".to_string()),
                            )
                            .await;
                            return;
                        }
                        bytes_downloaded += bytes.len() as u64;
                        set_status_and_persist(
                            &ctx.downloads,
                            &ctx.store_path,
                            &ctx.download_id,
                            DownloadStatus::InProgress {
                                bytes_downloaded,
                                total_bytes,
                            },
                        )
                        .await;
                    }
                    Some(Err(e)) => {
                        set_status_and_persist(
                            &ctx.downloads,
                            &ctx.store_path,
                            &ctx.download_id,
                            DownloadStatus::Failed(e.to_string()),
                        )
                        .await;
                        return;
                    }
                    None => {
                        set_status_and_persist(
                            &ctx.downloads,
                            &ctx.store_path,
                            &ctx.download_id,
                            DownloadStatus::Complete(dest_path.as_str().to_string()),
                        )
                        .await;
                        return;
                    }
                }
            }
        }
    }
}

async fn set_status_and_persist(
    downloads: &Arc<RwLock<HashMap<String, DownloadState>>>,
    store_path: &Utf8PathBuf,
    id: &str,
    status: DownloadStatus,
) {
    {
        let mut dl = downloads.write().await;
        if let Some(state) = dl.get_mut(id) {
            state.status = status;
            state.updated_at = OffsetDateTime::now_utc();
        }
    }
    if let Err(e) = persist_inner(downloads, store_path).await {
        warn!(error = %e, "failed to persist download status");
    }
}

async fn persist_inner(
    downloads: &Arc<RwLock<HashMap<String, DownloadState>>>,
    store_path: &Utf8PathBuf,
) -> Result<(), DownloadError> {
    let map = downloads.read().await;
    let store = DownloadStore {
        version: 1,
        downloads: map
            .iter()
            .map(|(id, state)| DownloadRecord {
                id: id.clone(),
                url: state.url.clone(),
                dest_path: state.dest_path.clone(),
                status: state.status.clone(),
                created_at: state.created_at,
                updated_at: state.updated_at,
                auth: state.auth.clone(),
            })
            .collect(),
    };
    drop(map);

    let json = serde_json::to_string_pretty(&store)?;
    let parent = store_path.parent().unwrap_or_else(|| Utf8Path::new("."));
    let tmp = NamedUtf8TempFile::new_in(parent)?;
    fs::tokio::write(tmp.path(), &json).await?;
    tmp.persist(store_path.as_std_path())
        .map_err(|e| DownloadError::Io(e.error))?;

    Ok(())
}
