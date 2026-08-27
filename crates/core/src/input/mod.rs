use crate::{messaging::ir::PrintingIR, types::ir::GlobalPrintingMeta};
use std::path::PathBuf;
use tempfile::TempDir;
pub mod zip;

/// Represents an abstraction around model for printing
pub struct Model {
    pub ir: Vec<PrintingIR>,
    pub model_meta: GlobalPrintingMeta,

    pub working_dir: TempDir,
    pub model_preview: Option<PathBuf>,
}
