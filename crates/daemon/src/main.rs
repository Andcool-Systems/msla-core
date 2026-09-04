mod broadcast;
mod input;
mod lcd;
mod peripheral;
mod printer_manager;
mod rest;
mod uart;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use msla_core::{
    config, logging,
    types::printer_manager::{PrinterCommand, PrinterState},
};
use tokio::sync::{Notify, mpsc, watch};
use tracing::{Level, error, info};

use crate::{
    broadcast::start_broadcast, lcd::LCDController, peripheral::PeripheralController,
    printer_manager::PrinterManager, rest::build_rest_api,
};

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

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
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");

        info!("Ctrl+C received");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logger
    let reload_handle = logging::init_logger(tracing::Level::DEBUG, true);

    // Load config from file
    config::load("./config.toml").await.map_err(|e| {
        error!("{}", e);
        std::process::exit(-1);
    });

    let config = config::get_config().await;

    logging::set_log_level(
        reload_handle,
        logging::str_to_log_level(&config.global.logging_level).unwrap_or(Level::INFO),
    );

    // Create printer command channel - main communication tunnel
    // between parts of code
    let (command_tx, command_rx) = mpsc::channel::<PrinterCommand>(128);

    // Global printer state
    let (state_tx, state_rx) = watch::channel(PrinterState::default());

    // Create peripheral controller - a bridge between high-level code
    // and middle- and low-level protocol
    let peripheral_controller = PeripheralController::new().await?;

    // Create lcd controller - wrapper around the linux framebuffer
    let lcd_controller = LCDController::new()?;

    // Build and run REST API
    let rest = build_rest_api(
        SocketAddrV4::new(
            Ipv4Addr::from_str(&config.rest_api.addr)?,
            config.rest_api.port,
        ),
        state_rx.clone(),
        command_tx.clone(),
    )?;

    tokio::spawn(rest);

    // Start broadcast server
    tokio::spawn(async { start_broadcast().await });

    // Some graceful shutdown things
    let shutdown_notify = Arc::new(Notify::new());

    // Graceful shutdown notifier task
    tokio::spawn({
        let shutdown_notify = shutdown_notify.clone();

        async move {
            shutdown_signal().await;
            shutdown_notify.notify_one();
        }
    });

    // Create printer manager instance and run it
    let mut printer =
        PrinterManager::new(command_rx, state_tx, peripheral_controller, lcd_controller);

    tokio::select! {
        _ = printer.run() => {},
        _ = shutdown_notify.notified() => {
            info!("Shutting down the server...");
        }
    }
    Ok(())
}
