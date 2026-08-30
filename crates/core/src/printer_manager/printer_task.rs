use tokio::{
    sync::{mpsc::Receiver, watch::Sender},
    time::sleep,
};
use tracing::{debug, error, info};

use crate::{
    input::Model,
    lcd::LCDController,
    messaging::ir::{MetaIR, PrintingIR},
    peripheral::PeripheralController,
    types::{
        peripheral::StepperPositioning,
        printer_manager::{PrinterTaskCommand, PrinterTaskState, PrintingError},
    },
};

/// Main printing task
pub struct PrinterTask {
    printing_model: Model,
    state: PrinterTaskState,

    command_receiver: Receiver<PrinterTaskCommand>,
    state_sender: Sender<PrinterTaskState>,

    peripheral_controller: PeripheralController,
    lcd_controller: LCDController,

    current_layer: u64,
}

impl PrinterTask {
    pub fn new(
        command_receiver: Receiver<PrinterTaskCommand>,
        state_sender: Sender<PrinterTaskState>,
        printing_model: Model,
        peripheral_controller: PeripheralController,
        lcd_controller: LCDController,
    ) -> Self {
        Self {
            command_receiver,
            state_sender,
            printing_model,
            peripheral_controller,
            lcd_controller,
            state: PrinterTaskState::Idle,
            current_layer: 0,
        }
    }

    pub async fn run(&mut self) {
        for i in 0..self.printing_model.ir.len() {
            let command = self.printing_model.ir[i].clone();

            match self.execute_next_step(command).await {
                Ok(Some(n)) => self.state = PrinterTaskState::Printing(n),
                Err(e) => {
                    self.send_status(PrinterTaskState::Error(e)).await;
                    self.state = PrinterTaskState::Paused(self.current_layer);
                },

                _ => {},
            }

            loop {
                if self.recv_command().await {
                    self.shutdown_peripherals().await;
                    return;
                }

                if !matches!(self.state, PrinterTaskState::Paused(_)) {
                    break;
                }
            }
        }
    }

    async fn execute_next_step(&self, command: PrintingIR) -> Result<Option<u64>, PrintingError> {
        match command {
            PrintingIR::Home => {
                debug!("Homing Z...");
                self.peripheral_controller
                    .home_z()
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot home Z axis: {e}")))?;
            },

            PrintingIR::MoveZ { pos, speed } => {
                debug!("Move Z to {}mm, speed: {}mm/m", pos, speed);
                self.peripheral_controller
                    .move_z_to(pos, speed, StepperPositioning::Absolute)
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot move Z axis: {e}")))?;
            },

            PrintingIR::TurnUV { state } => {
                debug!("Turn {} UV", if state { "on" } else { "off" });
                self.peripheral_controller
                    .turn_uv(state)
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot turn UV: {e}")))?;
            },

            PrintingIR::ShowImage(path_buf) => {
                self.lcd_controller
                    .show_image(self.printing_model.working_dir.path().join(path_buf))
                    .map_err(|e| PrintingError::new(format!("Cannot display image: {e}")))?;
            },

            PrintingIR::Wait(duration) => {
                info!("Sleep {:?}", duration);
                sleep(duration).await;
            },

            PrintingIR::DisableSteppers => {
                info!("Disable steppers");
                self.peripheral_controller
                    .disable_steppers()
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot disable steppers: {e}")))?;
            },

            PrintingIR::Meta(meta) => match meta {
                MetaIR::LayerStart(n) => {
                    return Ok(Some(n));
                },
                MetaIR::LayerEnd => {},
            },
        }

        Ok(None)
    }

    /// Send status to printing manager
    async fn send_status(&self, status: PrinterTaskState) {
        match self.state_sender.send(status) {
            Ok(_) => {},
            Err(e) => error!("Error sending status: {}", e),
        }
    }

    /// Receive external command
    async fn recv_command(&mut self) -> bool {
        match self.command_receiver.recv().await {
            Some(command) => match command {
                PrinterTaskCommand::Abort => return true,
                PrinterTaskCommand::Pause => {
                    self.state = PrinterTaskState::Paused(self.current_layer);
                    self.shutdown_peripherals().await;
                },
                PrinterTaskCommand::Resume => {
                    self.state = PrinterTaskState::Printing(self.current_layer);
                },
            },
            None => {
                error!("Printer Task has lost external control!");
                return true;
            },
        }

        false
    }

    /// Stop or pause peripherals
    async fn shutdown_peripherals(&mut self) {
        let _ = self.peripheral_controller.turn_uv(false).await;
        let _ = self
            .peripheral_controller
            .move_z_to(15f64, 45f64, StepperPositioning::Relative)
            .await;

        // other code
    }
}
