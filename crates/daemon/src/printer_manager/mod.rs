pub mod printer_task;

use crate::{
    lcd::LCDController,
    peripheral::PeripheralController,
    printer_manager::printer_task::PrinterTask,
};
use msla_core::types::{
    model::Model,
    peripheral::StepperPositioning,
    printer_manager::{PrinterCommand, PrinterState, PrintingError},
};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    watch::{self, Receiver as ReceiverWatch, Sender as WatchSender},
};
use tracing::{error, info};

pub struct PrinterManager {
    /// Current printing state
    state: PrinterState,

    /// Command receiver for manager
    command_receiver: Receiver<PrinterCommand>,

    /// State transmitter for other control sources
    state_transmitter: WatchSender<PrinterState>,

    print_task_command_transmitter: Option<Sender<PrinterCommand>>,
    print_task_state_receiver: Option<ReceiverWatch<PrinterState>>,

    peripheral_controller: PeripheralController,
    lcd_controller: LCDController,
}

impl PrinterManager {
    /// Create new printer instance
    pub fn new(
        receiver: Receiver<PrinterCommand>,
        transmitter: WatchSender<PrinterState>,
        per: PeripheralController,
        lcd: LCDController,
    ) -> Self {
        Self {
            state: PrinterState::Idle,
            command_receiver: receiver,
            state_transmitter: transmitter,
            peripheral_controller: per,
            lcd_controller: lcd,

            print_task_command_transmitter: None,
            print_task_state_receiver: None,
        }
    }

    /// Run printer manager instance
    pub async fn run(&mut self) {
        info!("Started Printer Manager");
        info!("Waiting for incoming commands...");

        loop {
            tokio::select! {
                Some(command) = self.command_receiver.recv() => {
                    match command {
                        PrinterCommand::StartPrint(model) => self.start_print(model),
                        PrinterCommand::Abort |
                        PrinterCommand::Pause |
                        PrinterCommand::Resume =>
                            self.send_to_print_task(command).await,

                        PrinterCommand::Home => {
                            let _ = self.peripheral_controller.home_z().await;
                        },
                        PrinterCommand::DisableStepper => {
                            let _ = self.peripheral_controller.disable_steppers().await;
                        },
                        PrinterCommand::MoveTo {pos, speed} => {
                            let _ = self.peripheral_controller.move_z_to(pos, speed, StepperPositioning::Absolute).await;
                        }
                    }
                }

                result = async {
                    match &mut self.print_task_state_receiver {
                        Some(receiver) => receiver.changed().await,
                        None => std::future::pending().await,
                    }
                } => {
                    match result {
                        Ok(()) => {
                            let event = self
                                .print_task_state_receiver
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .clone();

                            match event {
                                PrinterState::Aborted => {
                                    info!("Print aborted");
                                    self.reset_state().await;
                                }
                                PrinterState::Error(ref printing_error) => {
                                    error!("{}", printing_error);
                                    self.reset_state().await;
                                },

                                _ => {}

                            }

                            self.state = event;
                            self.send_status().await;
                        }

                        Err(_) => {
                            // PrinterTask destroyed Sender
                            self.print_task_state_receiver = None;
                        }
                    }
                }
            }
        }
    }

    /// Create Printer Task and start printing
    fn start_print(&mut self, model: Arc<Model>) {
        let (command_tx, command_rx) = mpsc::channel::<PrinterCommand>(128);
        let (state_tx, state_rx) = watch::channel(PrinterState::Idle);

        self.print_task_command_transmitter = Some(command_tx);
        self.print_task_state_receiver = Some(state_rx);

        let per = self.peripheral_controller.clone();
        let lcd = self.lcd_controller.clone();
        tokio::spawn(async move {
            let mut task = PrinterTask::new(state_tx, model, per, lcd);
            task.run(command_rx).await;
        });
    }

    /// Clear all communications with print task
    async fn reset_state(&mut self) {
        self.peripheral_controller.uart.reset_client().await;
        self.print_task_command_transmitter = None;
        self.print_task_state_receiver = None;
    }

    /// Send command into printing task
    async fn send_to_print_task(&mut self, task: PrinterCommand) {
        if let Some(tx) = &self.print_task_command_transmitter {
            let _ = tx.send(task).await.map_err(|e| {
                error!("Cannot send to a print task: {e}");
                self.state =
                    PrinterState::Error(PrintingError::new("Cannot send to a print task: {e}"))
            });
        }
    }

    /// Send current status to all external listeners
    async fn send_status(&mut self) {
        let _ = self.state_transmitter.send(self.state.clone());
    }
}
