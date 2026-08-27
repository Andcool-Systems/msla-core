mod config;
mod input;
mod lcd;
mod logging;
mod messaging;
mod peripheral;
mod types;
mod uart;

use std::env::args;

use anyhow::Result;
use tokio::time::sleep;
use tracing::{Level, error, info};

use crate::{
    input::zip::load_zip_model,
    lcd::LCDController,
    messaging::ir::{MetaIR, PrintingIR},
    peripheral::PeripheralController,
};

fn format_duration(total_seconds: u32) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    match (hours, minutes) {
        (h, _) if h > 0 => format!("{h}h {minutes}m {seconds}s"),
        (_, m) if m > 0 => format!("{m}m {seconds}c"),
        _ => format!("{seconds}c"),
    }
}

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

    let binding = args().collect::<Vec<String>>();

    let model_name = match binding.get(1) {
        Some(n) => n,
        None => {
            error!("Provide model to print");
            std::process::exit(-1);
        },
    };

    let f = async || -> Result<()> {
        let model = load_zip_model(model_name).await?;

        info!("Welcome to MSLA LCD!");
        info!("Printing file: {}", model_name);
        info!(
            "Estimated printing time: {} ({} layers)",
            format_duration(model.model_meta.estimated_printing_time),
            model.model_meta.total_layer_count
        );

        let controller = PeripheralController::new().await?;
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
                        n,
                        (n as f64 / model.model_meta.total_layer_count as f64) * 100.0
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

    Ok(())
}
