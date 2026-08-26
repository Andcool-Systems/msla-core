use crate::messaging::ir::PrintingIR;
use std::path::PathBuf;
use tempfile::TempDir;
pub mod zip;

/// Represents an abstraction around model for printing
#[derive(Debug)]
pub struct Model {
    pub ir: Vec<PrintingIR>,

    pub working_dir: TempDir,
    pub model_preview: Option<PathBuf>,
}
