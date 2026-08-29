mod config;
mod input;
mod lcd;
mod logging;
mod messaging;
mod peripheral;
mod printer_manager;
mod types;
mod uart;

use std::env::args;

use anyhow::Result;
use tokio::sync::{mpsc, watch};
use tracing::{Level, error};

use crate::{
    input::zip::load_zip_model,
    lcd::LCDController,
    peripheral::PeripheralController,
    printer_manager::PrinterManager,
    types::printer_manager::{PrinterCommand, PrinterState},
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

    // Create printer manager instance and run it
    let mut printer =
        PrinterManager::new(command_rx, state_tx, peripheral_controller, lcd_controller);
    tokio::spawn(async move { printer.run().await });

    // --------------- START OF INDEV CODE -------------------

    let binding = args().collect::<Vec<String>>();

    let model_name = match binding.get(1) {
        Some(n) => n,
        None => {
            error!("Provide model to print");
            std::process::exit(-1);
        },
    };

    let model = load_zip_model(model_name).await?;

    command_tx.send(PrinterCommand::StartPrint(model)).await?;

    let mut last = PrinterState::Idle;
    loop {
        let x = state_rx.borrow().clone();

        if last != x {
            println!("{:?}", x);
            last = x;
        }
    }

    /*

    let f = async || -> Result<()> {
        let model = load_zip_model(model_name).await?;

        info!("Welcome to MSLA LCD!");
        info!("Printing file: {}", model_name);
        info!(
            "Estimated printing time: {} ({} layers)",
            format_duration(model.model_meta.estimated_printing_time.unwrap_or_default()),
            model.model_meta.total_layer_count.unwrap_or_default()
        );
        let lcd = LCDController::new()?;

        for command in model.ir {
            match command {
                PrintingIR::Home => {
                    info!("homing z");
                    controller.home_z().await?;
                },
                PrintingIR::MoveZ { pos, speed } => {
                    info!("move z to {}", pos);
                    controller.move_z_to(pos, speed).await?;
                },
                PrintingIR::TurnUV { state } => {
                    info!("turn uv to {}", state);
                    controller.turn_uv(state).await?;
                },
                PrintingIR::ShowImage(path_buf) => {
                    lcd.show_image(model.working_dir.path().join(path_buf))?;
                },
                PrintingIR::Wait(duration) => {
                    info!("sleep {:?}", duration);
                    sleep(duration).await;
                },
                PrintingIR::DisableSteppers => {
                    info!("Disable steppers");
                    controller.disable_steppers().await?;
                },

                PrintingIR::Meta(meta) => match meta {
                    MetaIR::LayerStart(n) => info!(
                        "------------------ Printing layer {} ({:.2}%) ------------------",
                        n + 1,
                        ((n + 1) as f64 / model.model_meta.total_layer_count.unwrap_or(0) as f64)
                            * 100.0
                    ),
                    MetaIR::LayerEnd => {
                        info!("------------------ Layer finished ------------------")
                    },
                },
            }
        }

        Ok(())
    };

    match f().await {
        Ok(_) => info!("Printing done! Goodbye <3"),
        Err(e) => error!("Internal error occurred: {}", e),
    }

    */
}
