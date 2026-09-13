use std::{sync::Arc, time::Instant};

use msla_core::{
    config,
    types::{
        model::{Model, ir::PrintingIR},
        peripheral::StepperPositioning,
        printer_manager::{PrinterTaskCommand, PrinterTaskState, PrintingError, PrintingTaskMeta},
    },
};
use tokio::{
    sync::{mpsc::Receiver, watch::Sender},
    time::sleep,
};
use tracing::{debug, error};

use crate::{lcd::LCDController, peripheral::PeripheralController};

/// Main printing task
pub struct PrinterTask {
    printing_model: Arc<Model>,
    current_layer: usize,
    current_ir_index: usize,
    current_ir_elapsed: Instant,
    total_elapsed: Instant,

    state: PrinterTaskState,

    state_sender: Sender<PrinterTaskState>,

    peripheral_controller: PeripheralController,
    lcd_controller: LCDController,
}

impl PrinterTask {
    pub fn new(
        state_sender: Sender<PrinterTaskState>,
        printing_model: Arc<Model>,
        peripheral_controller: PeripheralController,
        lcd_controller: LCDController,
    ) -> Self {
        Self {
            state_sender,
            printing_model,
            peripheral_controller,
            lcd_controller,
            state: PrinterTaskState::Idle,
            current_layer: 0,
            current_ir_index: 0,
            current_ir_elapsed: Instant::now(),
            total_elapsed: Instant::now(),
        }
    }

    pub async fn run(&mut self, mut command_receiver: Receiver<PrinterTaskCommand>) {
        self.total_elapsed = Instant::now();
        self.state = PrinterTaskState::Printing(self.build_task_meta());

        for i in 0..self.printing_model.ir.len() {
            self.current_ir_index = i;
            self.current_ir_elapsed = Instant::now();
            self.send_current_status().await;

            match self.state {
                PrinterTaskState::Aborted | PrinterTaskState::Error(_) => return,
                PrinterTaskState::Paused(_) => {
                    if !self.wait_to_resume(&mut command_receiver).await {
                        self.shutdown_peripherals();
                        self.send_current_status().await;
                        return;
                    }
                },
                _ => {},
            }

            let command = self.printing_model.ir[i].clone();
            tokio::select! {
                result = self.execute_next_step(command.ir) => {
                    match result {
                        Ok(_) => {
                            self.state = PrinterTaskState::Printing(
                                self.build_task_meta()
                            );
                        }
                        Err(e) => self.state = PrinterTaskState::Error(e),
                    }
                }

                command = command_receiver.recv() => {
                    match command {
                        Some(command) => {
                            if !self.handle_command(command).await {
                                self.send_current_status().await;
                                return;
                            }
                        }

                        None => {
                            error!("Printer Task has lost external control!");
                            return;
                        }
                    }
                }
            }
        }

        self.state = PrinterTaskState::Finished;
        self.send_current_status().await;
    }

    async fn execute_next_step(&mut self, command: PrintingIR) -> Result<(), PrintingError> {
        match command {
            PrintingIR::Home => {
                debug!("Homing Z...");
                self.peripheral_controller
                    .home_z()
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot home Z axis: {e}")))?;
            },

            PrintingIR::MoveZ(mov) => {
                debug!("Move Z to {}mm, speed: {}mm/m", mov.pos, mov.speed);
                self.peripheral_controller
                    .move_z_to(mov.pos, mov.speed, StepperPositioning::Absolute)
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
                self.current_layer += 1;
                self.lcd_controller
                    .show_image(self.printing_model.working_dir.path().join(path_buf))
                    .map_err(|e| PrintingError::new(format!("Cannot display image: {e}")))?;
            },

            PrintingIR::Wait(duration) => {
                debug!("Sleep {:?}", duration);
                sleep(duration).await;
            },

            PrintingIR::DisableSteppers => {
                debug!("Disable steppers");
                self.peripheral_controller
                    .disable_steppers()
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot disable steppers: {e}")))?;
            },

            PrintingIR::EnableSteppers => {
                debug!("Enable steppers");
                self.peripheral_controller
                    .enable_steppers()
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot enable steppers: {e}")))?;
            },

            PrintingIR::SetStepperCurrent(current) => {
                debug!("Set stepper current: {}mA", current);
                self.peripheral_controller
                    .set_motor_current(current)
                    .await
                    .map_err(|e| PrintingError::new(format!("Cannot set stepper current: {e}")))?;
            },
        }

        Ok(())
    }

    /// Send status to printing manager
    async fn send_current_status(&self) {
        match self.state_sender.send(self.state.clone()) {
            Ok(_) => {},
            Err(e) => error!("Error sending status: {}", e),
        }
    }

    /// Stop or pause peripherals
    fn shutdown_peripherals(&mut self) {
        let peripheral_controller = self.peripheral_controller.clone();
        tokio::spawn(async move {
            let config = config::get_config().await;
            // Disable UV
            let _ = peripheral_controller.turn_uv(false).await;

            // Tear off the printed material from the film
            let _ = peripheral_controller
                .move_z_to(5f64, 40f64, StepperPositioning::Relative)
                .await;

            // QWe quickly rise to the topuickly rise to the top
            let _ = peripheral_controller
                .move_z_to(
                    config.physical.machine_height as f64,
                    300f64,
                    StepperPositioning::Absolute,
                )
                .await;

            // And... Disable stepper!
            let _ = peripheral_controller.disable_steppers().await;
        });
    }

    /// Blocks current task until it's resumed or aborted
    ///
    /// Returns `false` if aborted, and `true` if resumed
    async fn wait_to_resume(
        &mut self,
        command_receiver: &mut Receiver<PrinterTaskCommand>,
    ) -> bool {
        loop {
            match command_receiver.recv().await {
                Some(c) => match c {
                    PrinterTaskCommand::Pause => {},
                    PrinterTaskCommand::Resume => {
                        self.state = PrinterTaskState::Printing(self.build_task_meta());
                        return true;
                    },
                    PrinterTaskCommand::Abort => {
                        self.state = PrinterTaskState::Aborted;
                        return false;
                    },
                },

                None => {
                    error!("Printer Task has lost external control!");
                    return false;
                },
            }
        }
    }

    /// Handle external command
    /// Returns `false` if print aborted,
    /// and `true` if resumed/paused
    async fn handle_command(&mut self, command: PrinterTaskCommand) -> bool {
        match command {
            PrinterTaskCommand::Abort => {
                self.state = PrinterTaskState::Aborted;
                self.shutdown_peripherals();
                false
            },

            PrinterTaskCommand::Pause => {
                self.state = PrinterTaskState::Paused(self.build_task_meta());
                let _ = self.peripheral_controller.turn_uv(false).await;
                true
            },

            PrinterTaskCommand::Resume => true,
        }
    }

    fn build_task_meta(&self) -> PrintingTaskMeta {
        PrintingTaskMeta {
            printing_layer: self.current_layer,
            current_ir_index: self.current_ir_index,
            current_ir_elapsed: self.current_ir_elapsed,
            total_elapsed: self.total_elapsed,
            model: self.printing_model.clone(),
        }
    }
}
