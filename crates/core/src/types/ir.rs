#[derive(Debug)]
pub struct GlobalPrintingMeta {
    pub file_name: String,
    pub total_layer_count: u32,
    pub estimated_printing_time: u32,
    pub volume: f32,
    pub weight: f32,
    pub price: f32,
    pub layer_height: f32,
}

impl GlobalPrintingMeta {
    pub fn new() -> Self {
        Self {
            file_name: String::new(),
            total_layer_count: 0,
            estimated_printing_time: 0,
            volume: 0.0,
            weight: 0.0,
            price: 0.0,
            layer_height: 0.0,
        }
    }
}
