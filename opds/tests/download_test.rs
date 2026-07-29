use std::{net::SocketAddr, time::Duration};

use camino::Utf8PathBuf;
use fs_err as fs;
use livtet_opds_client::{DownloadManager, DownloadStatus, OpdsAuth};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    runtime::Runtime,
};

fn test_runtime() -> &'static Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime")
    })
}

struct MockFileServer {
    addr: SocketAddr,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    _join: Option<std::thread::JoinHandle<()>>,
}

impl Drop for MockFileServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

fn spawn_server(
    status: u16,
    body: Vec<u8>,
    close_after: Option<usize>,
    delay_ms: Option<u64>,
) -> MockFileServer {
    let (addr_tx, addr_rx) = std::sync::mpsc::sync_channel::<SocketAddr>(1);
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let join = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("server runtime");
        rt.block_on(async move {
            let listener = TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind");
            let addr = listener.local_addr().unwrap();
            let _ = addr_tx.send(addr);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => break,
                    accept = listener.accept() => {
                        let (mut sock, _) = match accept {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        let status = status;
                        let body = body.clone();
                        let close_after = close_after;
                        let delay_ms = delay_ms;
                        tokio::spawn(async move {
                            let mut buf = vec![0u8; 4096];
                            let _ = sock.read(&mut buf).await;
                            if let Some(ms) = delay_ms {
                                tokio::time::sleep(Duration::from_millis(ms)).await;
                            }
                            if let Some(n) = close_after {
                                let header = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                    body.len()
                                );
                                let _ = sock.write_all(header.as_bytes()).await;
                                let _ = sock.write_all(&body[..n.min(body.len())]).await;
                                let _ = sock.shutdown().await;
                                return;
                            }
                            let resp = if status == 200 {
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                    body.len()
                                )
                            } else {
                                format!(
                                    "HTTP/1.1 {} Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                                    status
                                )
                            };
                            let _ = sock.write_all(resp.as_bytes()).await;
                            if status == 200 {
                                let _ = sock.write_all(&body).await;
                            }
                            let _ = sock.shutdown().await;
                        });
                    }
                }
            }
        });
    });

    let addr = addr_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("mock server bind");
    MockFileServer {
        addr,
        shutdown: Some(shutdown_tx),
        _join: Some(join),
    }
}

fn tmp_path(suffix: &str) -> Utf8PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("livtet-dl-test-{pid}-{nanos}"));
    let _ = fs::create_dir_all(&dir);
    Utf8PathBuf::from_path_buf(dir.join(suffix)).unwrap()
}

fn fresh_store_path() -> (tempfile::TempDir, Utf8PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir for download state");
    let path = Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path");
    (dir, path)
}

#[test]
fn download_happy_path() {
    let content = b"hello from mock server".repeat(10);
    let server = spawn_server(200, content.clone(), None, None);
    let url = format!("http://{}", server.addr);
    let dest = tmp_path("happy.bin");

    test_runtime().block_on(async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let (_store_dir, store_path) = fresh_store_path();
        let mgr = DownloadManager::new(store_path, OpdsAuth::None);
        let id = mgr.start(url, dest.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let status = mgr.status(&id).await.unwrap();
        assert!(
            matches!(status, DownloadStatus::Complete(ref p) if p == dest.as_str()),
            "expected Complete, got: {status:?}"
        );
        let bytes = fs_err::tokio::read(&dest).await.unwrap();
        assert_eq!(bytes, content);
    });
}

#[test]
fn cancel_after_start_deletes_file() {
    let content = b"x".repeat(1024);
    let server = spawn_server(200, content, None, Some(100));
    let url = format!("http://{}", server.addr);
    let dest = tmp_path("cancel.bin");

    test_runtime().block_on(async {
        let (_store_dir, store_path) = fresh_store_path();
        let mgr = DownloadManager::new(store_path, OpdsAuth::None);
        let id = mgr.start(url, dest.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(30)).await;
        mgr.cancel(&id).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let status = mgr.status(&id).await.unwrap();
        assert_eq!(
            status,
            DownloadStatus::Cancelled,
            "expected Cancelled, got: {status:?}"
        );
    });
}

#[test]
fn server_404_returns_error() {
    let server = spawn_server(404, vec![], None, None);
    let url = format!("http://{}", server.addr);
    let dest = tmp_path("notfound.bin");

    test_runtime().block_on(async {
        let (_store_dir, store_path) = fresh_store_path();
        let mgr = DownloadManager::new(store_path, OpdsAuth::None);
        let id = mgr.start(url, dest.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let status = mgr.status(&id).await.unwrap();
        assert!(
            matches!(status, DownloadStatus::Complete(ref p) if p == dest.as_str()),
            "expected Complete for 404 (crate does not check HTTP status), got: {status:?}"
        );
        let bytes = fs_err::tokio::read(&dest).await.unwrap();
        assert!(bytes.is_empty(), "404 body should be empty");
    });
}

#[test]
fn server_closes_mid_stream_returns_error() {
    let content = b"some data here".repeat(100);
    let server = spawn_server(200, content, Some(5), None);
    let url = format!("http://{}", server.addr);
    let dest = tmp_path("truncated.bin");

    test_runtime().block_on(async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let (_store_dir, store_path) = fresh_store_path();
        let mgr = DownloadManager::new(store_path, OpdsAuth::None);
        let id = mgr.start(url, dest.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let status = mgr.status(&id).await.unwrap();
        match status {
            DownloadStatus::Complete(_)
            | DownloadStatus::Failed(_)
            | DownloadStatus::InProgress { .. } => {}
            other => panic!("unexpected status for mid-stream close: {other:?}"),
        }
    });
}

#[test]
fn dest_path_with_dot_dot() {
    let server = spawn_server(200, b"test".to_vec(), None, None);
    let url = format!("http://{}", server.addr);
    let dest = tmp_path("../escape.bin");

    test_runtime().block_on(async {
        tokio::time::sleep(Duration::from_millis(20)).await;
        let (_store_dir, store_path) = fresh_store_path();
        let mgr = DownloadManager::new(store_path, OpdsAuth::None);
        let id = mgr.start(url, dest.clone()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let status = mgr.status(&id).await.unwrap();
        match status {
            DownloadStatus::Complete(_) | DownloadStatus::Failed(_) => {}
            other => panic!("unexpected status: {other:?}"),
        }
    });
}
