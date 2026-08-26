mod config;
mod input;
mod lcd;
mod logging;
mod messaging;
mod peripheral;
mod types;
mod uart;

use anyhow::Result;
use tokio::time::sleep;
use tracing::{Level, error, info};

use crate::{
    input::zip::load_zip_model, lcd::LCDController, messaging::ir::PrintingIR,
    peripheral::PeripheralController,
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

    let f = async || -> Result<()> {
        let model = load_zip_model("model.zip").await?;
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
            }
        }

        Ok(())
    };

    match f().await {
        Ok(_) => {},
        Err(e) => error!("Internal error occurred: {}", e),
    }

    Ok(())
}
