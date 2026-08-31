use std::{path::PathBuf, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum PrintingIR {
    Home,

    MoveZ { pos: f64, speed: f64 },
    TurnUV { state: bool },

    ShowImage(PathBuf),
    Wait(Duration),

    DisableSteppers,
    EnableSteppers,

    Meta(MetaIR),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetaIR {
    LayerStart(u64),
    LayerEnd,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalPrintingMeta {
    pub file_name: Option<String>,
    pub total_layer_count: Option<u64>,
    pub estimated_printing_time: Option<u32>,
    pub volume: Option<f32>,
    pub weight: Option<f32>,
    pub price: Option<f32>,
    pub layer_height: Option<f32>,
}

impl GlobalPrintingMeta {
    pub fn new() -> Self {
        Self {
            file_name: None,
            total_layer_count: None,
            estimated_printing_time: None,
            volume: None,
            weight: None,
            price: None,
            layer_height: None,
        }
    }
}
