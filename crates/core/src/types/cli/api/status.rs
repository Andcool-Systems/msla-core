use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StatusResponse {
    pub state: String,
    pub current_status: Option<CurrentStatusResponse>,
    pub model_meta: Option<ModelMetaResponse>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CurrentStatusResponse {
    pub current_ir_index: usize,
    pub current_layer: usize,
    pub estimated_finish_time: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ModelMetaResponse {
    pub estimated_printing_time: usize,
    pub ir_len: usize,
    pub layer_height: f64,
    pub name: String,
    pub price: f64,
    pub total_layer_count: usize,
    pub volume: f64,
    pub weight: f64,
}
