use std::{net::SocketAddr, sync::Arc};

use livtet_opds_types::OpdsError;
use poem::{
    Server,
    listener::{Listener, TcpListener},
};
use tokio::{sync::watch, task::JoinHandle};

use crate::{SharedOpdsHandler, auth::AuthConfig, handler::OpdsServer, routes::make_routes};

pub struct PoemOpdsServer {
    handler: SharedOpdsHandler,
    auth_config: Option<AuthConfig>,
    addr: Option<SocketAddr>,
    shutdown_tx: Option<watch::Sender<bool>>,
    join: Option<JoinHandle<std::io::Result<()>>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl PoemOpdsServer {
    pub fn new(handler: SharedOpdsHandler) -> Self {
        Self {
            handler,
            auth_config: None,
            addr: None,
            shutdown_tx: None,
            join: None,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Set the authentication configuration for this server instance.
    ///
    /// Must be called before `start()` for the auth middleware to be
    /// wired into the route table.
    pub fn set_auth_config(&mut self, config: AuthConfig) {
        self.auth_config = Some(config);
    }

    pub fn auth_config(&self) -> Option<&AuthConfig> {
        self.auth_config.as_ref()
    }

    pub fn bound_addr(&self) -> Option<SocketAddr> {
        self.addr
    }
}

#[async_trait::async_trait]
impl OpdsServer for PoemOpdsServer {
    async fn start(&mut self, addr: SocketAddr) -> Result<(), OpdsError> {
        if self.running.load(std::sync::atomic::Ordering::Acquire) {
            tracing::debug!(%addr, "OPDS server already running; start() is a no-op");
            return Ok(());
        }

        let listener = TcpListener::bind(addr);
        let acceptor = listener
            .into_acceptor()
            .await
            .map_err(|e| OpdsError::Network(format!("bind {}: {}", addr, e)))?;

        let app = make_routes(self.handler.clone(), self.auth_config.clone());

        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let running = Arc::clone(&self.running);

        let join: JoinHandle<std::io::Result<()>> = tokio::spawn(async move {
            let signal = async move {
                while shutdown_rx.changed().await.is_ok() {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            };
            let result = Server::new_with_acceptor(acceptor)
                .name("livtet-opds")
                .run_with_graceful_shutdown(app, signal, Some(std::time::Duration::from_secs(5)))
                .await;
            running.store(false, std::sync::atomic::Ordering::Release);
            if let Err(ref e) = result {
                tracing::error!(error = %e, "OPDS server task exited with error");
            } else {
                tracing::info!("OPDS server shut down cleanly");
            }
            result
        });

        self.addr = Some(addr);
        self.shutdown_tx = Some(shutdown_tx);
        self.join = Some(join);
        self.running
            .store(true, std::sync::atomic::Ordering::Release);
        tracing::info!(%addr, "OPDS server started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), OpdsError> {
        if !self.running.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(());
        }

        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }

        if let Some(join) = self.join.take() {
            match join.await {
                Ok(Ok(())) => {
                    tracing::info!("OPDS server stopped cleanly");
                }
                Ok(Err(e)) => {
                    return Err(OpdsError::Network(format!(
                        "server error during shutdown: {}",
                        e
                    )));
                }
                Err(join_err) if join_err.is_panic() => {
                    return Err(OpdsError::Network("OPDS server task panicked".to_string()));
                }
                Err(join_err) => {
                    return Err(OpdsError::Network(format!(
                        "OPDS server join error: {}",
                        join_err
                    )));
                }
            }
        }

        self.addr = None;
        self.running
            .store(false, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Acquire)
    }
}
