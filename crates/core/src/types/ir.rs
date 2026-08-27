pub struct GlobalPrintingMeta {
    pub total_layer_count: u32,
}

impl GlobalPrintingMeta {
    pub fn new() -> Self {
        Self {
            total_layer_count: 0,
        }
    }
}
