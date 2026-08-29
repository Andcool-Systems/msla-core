#[derive(Debug)]
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
