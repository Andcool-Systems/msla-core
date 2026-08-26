mod config;
mod input;
mod lcd;
mod messaging;
mod peripheral;
mod types;
mod uart;

use anyhow::Result;
use tokio::time::sleep;

use crate::{
    input::zip::load_zip_model, lcd::LCDController, messaging::ir::PrintingIR,
    peripheral::PeripheralController,
};

#[tokio::main]
async fn main() -> Result<()> {
    config::load("./config.toml").await?;

    let model = load_zip_model("model.zip").await.unwrap();
    let controller = PeripheralController::new().await.unwrap();
    let lcd = LCDController::new()?;

    for command in model.ir {
        match command {
            PrintingIR::Home => {
                println!("homing z");
                controller.home_z().await?;
            },
            PrintingIR::MoveZ { pos, speed } => {
                println!("move z to {}", pos);
                controller.move_z_to(pos, speed).await?;
            },
            PrintingIR::TurnUV { state } => {
                println!("turn uv to {}", state);
                controller.turn_uv(state).await?;
            },
            PrintingIR::ShowImage(path_buf) => {
                lcd.show_image(model.working_dir.path().join(path_buf))?;
            },
            PrintingIR::Wait(duration) => {
                println!("sleep {:?}", duration);
                sleep(duration).await;
            },
            PrintingIR::DisableSteppers => {},
        }
    }

    Ok(())
}
