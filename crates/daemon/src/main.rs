mod config;
mod input;
mod lcd;
mod logging;
mod peripheral;
mod printer_manager;
mod rest;
mod uart;

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use anyhow::Result;
use msla_core::types::printer_manager::{PrinterCommand, PrinterState};
use tokio::sync::{mpsc, watch};
use tracing::{Level, error};

use crate::{
    lcd::LCDController, peripheral::PeripheralController, printer_manager::PrinterManager,
    rest::build_rest_api,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up logger
    let reload_handle = logging::init_logger(tracing::Level::DEBUG);

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

    // Create printer manager instance and run it
    let mut printer =
        PrinterManager::new(command_rx, state_tx, peripheral_controller, lcd_controller);
    printer.run().await;

    Ok(())
}
