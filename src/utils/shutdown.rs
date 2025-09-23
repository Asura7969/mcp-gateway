use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Graceful shutdown coordinator
#[derive(Clone)]
pub struct ShutdownCoordinator {
    /// Shutdown signal sender
    shutdown_tx: watch::Sender<bool>,
    /// Active connections counter
    active_connections: Arc<AtomicUsize>,
    /// Shutdown in progress flag
    shutdown_in_progress: Arc<AtomicBool>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            shutdown_tx,
            active_connections: Arc::new(AtomicUsize::new(0)),
            shutdown_in_progress: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a shutdown receiver to listen for shutdown signals
    pub fn subscribe(&self) -> watch::Receiver<bool> {
        self.shutdown_tx.subscribe()
    }

    /// Increment active connections
    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrement active connections
    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::SeqCst);
    }

    /// Get current active connections count
    pub fn active_connections_count(&self) -> usize {
        self.active_connections.load(Ordering::SeqCst)
    }

    /// Check if shutdown is in progress
    pub fn is_shutdown_in_progress(&self) -> bool {
        self.shutdown_in_progress.load(Ordering::SeqCst)
    }

    /// Start graceful shutdown
    pub async fn start_graceful_shutdown(&self, timeout_duration: Duration) {
        info!("Starting graceful shutdown process");

        // Mark shutdown as in progress
        self.shutdown_in_progress.store(true, Ordering::SeqCst);

        // Send shutdown signal to all subscribers
        if let Err(e) = self.shutdown_tx.send(true) {
            error!("Failed to send shutdown signal: {}", e);
        }

        // Wait for active connections to complete with timeout
        let start_time = std::time::Instant::now();
        loop {
            let active = self.active_connections_count();
            if active == 0 {
                info!("All connections have been closed gracefully");
                break;
            }

            if start_time.elapsed() >= timeout_duration {
                warn!(
                    "Graceful shutdown timeout reached. {} connections still active",
                    active
                );
                break;
            }

            info!("Waiting for {} active connections to close...", active);
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        info!("Graceful shutdown completed");
    }

    /// Force shutdown immediately
    pub fn force_shutdown(&self) {
        warn!("Force shutdown initiated");
        self.shutdown_in_progress.store(true, Ordering::SeqCst);
        if let Err(e) = self.shutdown_tx.send(true) {
            error!("Failed to send force shutdown signal: {}", e);
        }
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for shutdown signal
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown");
        },
    }
}

/// Create a graceful shutdown handler with timeout
pub async fn graceful_shutdown_with_timeout(
    coordinator: ShutdownCoordinator,
    timeout_duration: Duration,
    force_shutdown: bool,
) {
    if force_shutdown {
        coordinator.force_shutdown();
    } else {
        coordinator.start_graceful_shutdown(timeout_duration).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_shutdown_coordinator_creation() {
        let coordinator = ShutdownCoordinator::new();
        assert_eq!(coordinator.active_connections_count(), 0);
        assert!(!coordinator.is_shutdown_in_progress());
    }

    #[tokio::test]
    async fn test_connection_tracking() {
        let coordinator = ShutdownCoordinator::new();

        // Test increment
        coordinator.increment_connections();
        assert_eq!(coordinator.active_connections_count(), 1);

        coordinator.increment_connections();
        assert_eq!(coordinator.active_connections_count(), 2);

        // Test decrement
        coordinator.decrement_connections();
        assert_eq!(coordinator.active_connections_count(), 1);

        coordinator.decrement_connections();
        assert_eq!(coordinator.active_connections_count(), 0);
    }

    #[tokio::test]
    async fn test_force_shutdown() {
        let coordinator = ShutdownCoordinator::new();
        assert!(!coordinator.is_shutdown_in_progress());

        coordinator.force_shutdown();
        assert!(coordinator.is_shutdown_in_progress());
    }

    #[tokio::test]
    async fn test_graceful_shutdown_with_no_connections() {
        let coordinator = ShutdownCoordinator::new();

        // Should complete immediately with no connections
        coordinator
            .start_graceful_shutdown(Duration::from_millis(100))
            .await;
        assert!(coordinator.is_shutdown_in_progress());
    }

    #[tokio::test]
    async fn test_graceful_shutdown_with_timeout() {
        let coordinator = ShutdownCoordinator::new();

        // Add some connections
        coordinator.increment_connections();
        coordinator.increment_connections();

        let start = std::time::Instant::now();

        // Should timeout after 100ms
        coordinator
            .start_graceful_shutdown(Duration::from_millis(100))
            .await;

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(90)); // 略低于预期，考虑系统延迟
        assert!(elapsed < Duration::from_millis(1000)); // 很宽松的上限
        assert!(coordinator.is_shutdown_in_progress());
    }
}
