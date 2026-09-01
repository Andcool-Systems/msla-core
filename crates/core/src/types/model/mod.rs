use crate::types::model::ir::PrintingIR;
use std::{path::PathBuf, sync::Arc};
use tempfile::TempDir;
pub mod ir;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GlobalPrintingMeta {
    pub file_name: Option<String>,
    pub total_layer_count: Option<usize>,
    pub estimated_printing_time: Option<usize>,
    pub volume: Option<f32>,
    pub weight: Option<f32>,
    pub price: Option<f32>,
    pub layer_height: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub ir: Vec<PrintingIR>,
    pub model_meta: GlobalPrintingMeta,

    pub working_dir: Arc<TempDir>,
    pub model_preview: Option<PathBuf>,
}
