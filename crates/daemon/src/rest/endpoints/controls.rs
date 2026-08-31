use actix_web::{HttpResponse, Responder, post, web};
use msla_core::types::{
    printer_manager::{PrinterCommand, PrinterState},
    rest::RESTPrinterState,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::input::zip::load_zip_model;

#[post("/abort")]
pub async fn abort_print(state: web::Data<RESTPrinterState>) -> impl Responder {
    match state.command_tx.send(PrinterCommand::Abort).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"message": "Cannot send abort signal"}))
        },
    }
}

#[derive(Deserialize, Serialize)]
struct StartLocalPrint {
    path: String,
}

/// Starting a new zip print from local file
#[post("/start/zip/local")]
pub async fn start_print(
    state: web::Data<RESTPrinterState>,
    body: web::Json<StartLocalPrint>,
) -> impl Responder {
    let model = match load_zip_model(body.path.clone()).await {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(json!({"message": format!("Cannot load .zip model: {}", e)}));
        },
    };

    match state.state.borrow().clone() {
        PrinterState::Printing(_) | PrinterState::Paused(_) => {
            return HttpResponse::Conflict().json(
                json!({"message": "Printer is still printing. Abort print before starting new!"}),
            );
        },
        _ => {},
    }

    match state
        .command_tx
        .send(PrinterCommand::StartPrint(model))
        .await
    {
        Ok(()) => HttpResponse::Created().json(json!({"message": "Success"})),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({"message": format!("Cannot send command to daemon: {}", e)})),
    }
}
