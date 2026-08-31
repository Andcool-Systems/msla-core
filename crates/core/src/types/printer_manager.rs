use crate::types::model::Model;
use core::fmt;
use std::{error::Error, sync::Arc};

/// Command for controlling printer manager
pub enum PrinterCommand {
    StartPrint(Arc<Model>),
    Pause,
    Resume,
    Abort,
}

/// Global printer state
#[derive(Clone, Debug, Default)]
pub enum PrinterState {
    #[default]
    Idle,
    Printing(PrintingTaskMeta),
    Paused(PrintingTaskMeta),
    Error(PrintingError),
    Aborted,
    Finished,
}

impl PrinterState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Printing(_) => "printing",
            Self::Paused(_) => "paused",
            Self::Error(_) => "error",
            Self::Aborted => "aborted",
            Self::Finished => "finished",
        }
    }
}

/// Outgoing commands for printer task
#[derive(Debug)]
pub enum PrinterTaskCommand {
    Pause,
    Resume,
    Abort,
}

/// Current print state metadata
#[derive(Clone, Debug)]
pub struct PrintingTaskMeta {
    pub printing_layer: u64,
    pub model: Arc<Model>,
}

impl PrintingTaskMeta {
    pub fn new(layer: u64, model: Arc<Model>) -> Self {
        Self {
            printing_layer: layer,
            model,
        }
    }
}

/// State of current printing task
#[derive(Clone, Debug)]
pub enum PrinterTaskState {
    /// Printing layer #
    Printing(PrintingTaskMeta),
    Paused(PrintingTaskMeta),
    Error(PrintingError),
    Aborted,
    Finished,
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
