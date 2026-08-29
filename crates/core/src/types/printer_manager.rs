use core::fmt;
use std::error::Error;

use crate::input::Model;

/// Command for controlling printer manager
pub enum PrinterCommand {
    StartPrint(Model),
    Pause,
    Resume,
    Abort,
}

/// Global printer state
#[derive(Clone, Debug, PartialEq, Default)]
pub enum PrinterState {
    #[default]
    Idle,
    Printing {
        layer_no: u64,
    },
    Paused {
        layer_no: u64,
    },
    Error(PrintingError),
}

/// Outgoing commands for printer task
#[derive(Debug)]
pub enum PrinterTaskCommand {
    Pause,
    Resume,
    Abort,
}

/// State of current printing task
#[derive(Clone, Debug)]
pub enum PrinterTaskState {
    /// Printing layer #
    Printing(u64),
    Error(PrintingError),
    Paused(u64),
    Idle,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrintingError {
    message: String,
}

impl PrintingError {
    /// Create new printing error instance
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PrintingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred due model printing: {}", self.message)
    }
}

impl Error for PrintingError {}
