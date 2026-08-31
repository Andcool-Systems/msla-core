use crate::types::model::ir::{GlobalPrintingMeta, PrintingIR};
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;
pub mod ir;

#[derive(Clone, Debug)]
pub struct Model {
    pub ir: Vec<PrintingIR>,
    pub model_meta: GlobalPrintingMeta,

    pub working_dir: Arc<TempDir>,
    pub model_preview: Option<PathBuf>,
}
