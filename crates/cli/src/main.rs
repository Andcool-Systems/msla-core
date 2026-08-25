use std::time::Duration;

use anyhow::Result;
use msla_core::{input::zip::load_zip_model, peripheral::PeripheralController};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(async move {
        println!("{:?}", load_zip_model("model.zip").await);
        /*
        let controller = PeripheralController::new().unwrap();

        let f = async || -> Result<()> {
            println!("Homing Z...");
            controller.home_z().await?;

            println!("Z homed!\nMove to 5.05");
            controller.move_z_to(5.05, 50.0).await?;

            println!("Move to 0.05");
            controller.move_z_to(0.05, 150.0).await?;

            println!("Printer at start pos, turning on UV");
            controller.turn_uv(true).await?;

            println!("UV turned on!\nWaiting 50s...");

            sleep(Duration::from_secs(5)).await;

            println!("Waited 50s.., Turning off UV");
            controller.turn_uv(false).await?;

            println!("UV turned off! Slowly raising to 150mm...");
            controller.move_z_to(150.0, 300.0).await?;

            println!("Z raised! Printing finished! Goodbye!");

            Ok(())
        };

        println!("{:?}", f().await);

        */
    });
    loop {}
}
