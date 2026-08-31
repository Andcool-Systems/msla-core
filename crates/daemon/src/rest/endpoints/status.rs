use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use msla_core::types::{printer_manager::PrinterState, rest::RESTPrinterState};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize, Default)]
struct ResponseFormat<'a> {
    pub state: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_status: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_meta: Option<Value>,
}

#[get("/status")]
pub async fn get_status(state: web::Data<RESTPrinterState>) -> impl Responder {
    let printer_state = state.state.borrow().clone();
    let mut res = ResponseFormat {
        state: Some(printer_state.as_str()),
        ..Default::default()
    };

    match printer_state {
        PrinterState::Printing(meta) | PrinterState::Paused(meta) => {
            res.current_status = Some(json!({
                "current_layer": meta.printing_layer,
                "current_ir_index": meta.current_ir_index
            }));

            res.model_meta = Some(json!({
                "name": meta.model.model_meta.file_name,
                "total_layer_count": meta.model.model_meta.total_layer_count,
                "estimated_printing_time": meta.model.model_meta.estimated_printing_time,
                "volume": meta.model.model_meta.volume,
                "weight": meta.model.model_meta.weight,
                "price": meta.model.model_meta.price,
                "layer_height": meta.model.model_meta.price,

                "ir_len": meta.model.ir.len()
            }))
        },

        PrinterState::Error(error) => {
            res.error = Some(error.to_string());
        },

        _ => {},
    }

    HttpResponse::Ok().json(res)
}

#[get("/preview")]
pub async fn get_preview(req: HttpRequest, state: web::Data<RESTPrinterState>) -> impl Responder {
    let meta = match state.state.borrow().clone() {
        PrinterState::Printing(meta) | PrinterState::Paused(meta) => meta,
        _ => {
            return HttpResponse::BadRequest().json(json!({"message": "Printer not printing!"}));
        },
    };

    let path = match &meta.model.model_preview {
        Some(path) => path,
        None => return HttpResponse::NotFound().json(json!({"message": "Preview not available"})),
    };

    let named_file = match NamedFile::open(path) {
        Ok(nf) => nf,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"message": format!("Cannot open preview file: {}", e)}));
        },
    };

    named_file.into_response(&req)
}
