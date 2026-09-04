use std::path::PathBuf;

use crate::input::zip::load_zip_model;
use actix_multipart::form::text::Text;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_web::{HttpResponse, Responder, post, web};
use msla_core::types::{
    printer_manager::{PrinterCommand, PrinterState},
    rest::RESTPrinterState,
};
use serde_json::json;

#[post("/abort")]
pub async fn abort_print(state: web::Data<RESTPrinterState>) -> impl Responder {
    match state.command_tx.send(PrinterCommand::Abort).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"message": "Cannot send abort signal"}))
        },
    }
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(limit = "100MB")]
    file: Option<TempFile>,

    local_file: Option<Text<String>>,
}

/// Starting a new zip print from local file
#[post("/start/{f_type}/{placing}")]
pub async fn start_print(
    path: web::Path<(String, String)>,
    state: web::Data<RESTPrinterState>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    match state.state.borrow().clone() {
        PrinterState::Printing(_) | PrinterState::Paused(_) => {
            return HttpResponse::Conflict().json(
                json!({"message": "Printer is still printing. Abort print before starting new!"}),
            );
        },
        _ => {},
    }

    let mut file_handler = None;
    let file = match path.1.as_str() {
        "remote" => {
            let Some(file) = form.file else {
                return HttpResponse::BadRequest()
                    .json(json!({"message": "You need to pass a file"}));
            };

            let p = file.file.path().to_path_buf();
            file_handler = Some(file);
            p
        },
        "local" => {
            let Some(path) = form.local_file else {
                return HttpResponse::BadRequest()
                    .json(json!({"message": "You need to specify file path"}));
            };
            PathBuf::from(path.0)
        },
        _ => return HttpResponse::BadRequest().json(json!({"message": "Invalid placing"})),
    };

    let model = match path.0.as_str() {
        "zip" => match load_zip_model(file).await {
            Ok(m) => m,
            Err(e) => {
                return HttpResponse::BadRequest().json(json!({"message": format!("{}", e)}));
            },
        },
        _ => return HttpResponse::BadRequest().json(json!({"message": "Unknown file format"})),
    };

    drop(file_handler);

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

#[post("/home")]
pub async fn home(state: web::Data<RESTPrinterState>) -> impl Responder {
    match state.state.borrow().clone() {
        PrinterState::Printing(_) | PrinterState::Paused(_) => {
            return HttpResponse::BadRequest().json(json!({"message": "Printer is printing now!"}));
        },

        _ => {},
    };

    match state.command_tx.send(PrinterCommand::Home).await {
        Ok(_) => HttpResponse::Created().json(json!({"message": "Home task signal sent"})),
        Err(_) => {
            HttpResponse::InternalServerError().json(json!({"message": "Cannot send home signal"}))
        },
    }
}

#[post("/disable-stepper")]
pub async fn dis_stepper(state: web::Data<RESTPrinterState>) -> impl Responder {
    match state.state.borrow().clone() {
        PrinterState::Printing(_) | PrinterState::Paused(_) => {
            return HttpResponse::BadRequest().json(json!({"message": "Printer is printing now!"}));
        },

        _ => {},
    };

    match state.command_tx.send(PrinterCommand::DisableStepper).await {
        Ok(_) => {
            HttpResponse::Created().json(json!({"message": "Disable stepper task signal sent"}))
        },
        Err(_) => HttpResponse::InternalServerError()
            .json(json!({"message": "Cannot send disable stepper signal"})),
    }
}
