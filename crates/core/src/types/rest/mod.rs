use crate::types::printer_manager::{PrinterCommand, PrinterState};
use tokio::sync::{mpsc::Sender, watch};

/// State of restful api
pub struct RESTPrinterState {
    pub state: watch::Receiver<PrinterState>,
    pub command_tx: Sender<PrinterCommand>,
}
