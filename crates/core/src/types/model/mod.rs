use crate::types::model::{analyzer::Analyzer, ir::TimedIR};
use std::{
    ops::{Add, AddAssign},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tempfile::TempDir;
pub mod analyzer;
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
    pub ir: Vec<TimedIR>,
    pub model_meta: GlobalPrintingMeta,

    pub working_dir: Arc<TempDir>,
    pub model_preview: Option<PathBuf>,
}

impl Model {
    /// Create new model
    pub fn new(
        ir: Vec<TimedIR>,
        model_meta: GlobalPrintingMeta,
        working_dir: Arc<TempDir>,
        model_preview: Option<PathBuf>,
    ) -> Self {
        let mut m = Self {
            ir,
            model_meta,
            working_dir,
            model_preview,
        };

        m.calc_estimated();
        m
    }

    pub fn calc_estimated(&mut self) {
        let mut analyzer = Analyzer::default();
        let mut dur = analyzer.calc_command_duration(&self.ir.last().unwrap().ir);

        for x in self.ir.iter_mut().rev() {
            x.estimated_remaining = dur;
            dur += analyzer.calc_command_duration(&x.ir);
        }
    }
}
