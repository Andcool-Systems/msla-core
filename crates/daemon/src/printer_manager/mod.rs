pub mod printer_task;

use std::sync::Arc;

use crate::{
    lcd::LCDController, peripheral::PeripheralController,
    printer_manager::printer_task::PrinterTask,
};
use msla_core::types::{
    model::Model,
    printer_manager::{PrinterCommand, PrinterState, PrinterTaskCommand, PrinterTaskState},
};
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

    print_task_command_transmitter: Option<Sender<PrinterTaskCommand>>,
    print_task_state_receiver: Option<ReceiverWatch<PrinterTaskState>>,

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
        loop {
            tokio::select! {
                Some(command) = self.command_receiver.recv() => {
                    match command {
                        PrinterCommand::StartPrint(model) => self.start_print(model),
                        PrinterCommand::Abort => self.send_to_print_task(PrinterTaskCommand::Abort).await,
                        _ => {},
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
                                PrinterTaskState::Printing(meta) =>
                                    self.state = PrinterState::Printing(meta),

                                PrinterTaskState::Paused(meta) =>
                                    self.state = PrinterState::Paused(meta),

                                PrinterTaskState::Idle =>
                                    self.state = PrinterState::Idle,

                                PrinterTaskState::Aborted => {
                                    info!("Print aborted");
                                    self.state = PrinterState::Aborted;
                                    self.clear_print_task();
                                }

                                 PrinterTaskState::Finished =>
                                    self.state = PrinterState::Finished,

                                PrinterTaskState::Error(printing_error) => {
                                    error!("{}", printing_error);
                                    self.state = PrinterState::Error(printing_error);
                                    self.clear_print_task();
                                },

                            }

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
        let (command_tx, command_rx) = mpsc::channel::<PrinterTaskCommand>(128);
        let (state_tx, state_rx) = watch::channel(PrinterTaskState::Idle);

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
    fn clear_print_task(&mut self) {
        self.print_task_command_transmitter = None;
        self.print_task_state_receiver = None;
    }

    /// Send command into printing task
    async fn send_to_print_task(&self, task: PrinterTaskCommand) {
        if let Some(tx) = &self.print_task_command_transmitter {
            let _ = tx
                .send(task)
                .await
                .map_err(|e| error!("Cannot send to a print task: {e}"));
        }
    }

    /// Send current status to all external listeners
    async fn send_status(&mut self) {
        let _ = self.state_transmitter.send(self.state.clone());
    }
}
