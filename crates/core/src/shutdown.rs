pub async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        use tracing::info;

        let mut sigterm =
            signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {
                info!("SIGTERM received");
            }

            _ = sigint.recv() => {
                info!("SIGINT received");
            }
        }
    }

    #[cfg(not(unix))]
    {
        use tracing::info;

        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");

        info!("Ctrl+C received");
    }
}
