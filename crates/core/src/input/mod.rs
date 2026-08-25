use crate::messaging::ir::PrintingIR;
use std::path::PathBuf;
use tempfile::TempDir;
pub mod zip;

/// Represents an abstraction around model for printing
#[derive(Debug)]
pub struct Model {
    ir: Vec<PrintingIR>,

    working_dir: TempDir,
    model_preview: Option<PathBuf>,
}
