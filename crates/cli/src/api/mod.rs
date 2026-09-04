use std::{path::PathBuf, sync::OnceLock};

use anyhow::{Result, anyhow};
use msla_core::{config, types::cli::api::status::StatusResponse};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use tracing::{error, info};

static CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client<'a>() -> &'a Client {
    CLIENT.get_or_init(|| Client::new())
}

/// Return rest api url
async fn get_api_url() -> String {
    let config = config::get_config().await;
    format!(
        "http://{}:{}",
        config.rest_api.external_addr, config.rest_api.port
    )
}

/// Get current printing status
pub async fn get_status() -> Result<StatusResponse> {
    let response = match get_client()
        .get(format!("{}/status", get_api_url().await))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            anyhow::bail!("Cannot fetch current status: {}", e)
        },
    };

    if !response.status().is_success() {
        anyhow::bail!(
            "Cannot fetch current status: ({}) {}",
            response.status().as_u16(),
            response.text().await?
        )
    }

    Ok(serde_json::from_str(&response.text().await?)?)
}

pub enum PlacingType {
    Local,
    Remote,
}

impl PlacingType {
    pub fn to_str(&self) -> &str {
        match self {
            PlacingType::Local => "local",
            PlacingType::Remote => "remote",
        }
    }
}

pub enum FileExt {
    Zip,
}

impl FileExt {
    pub fn to_str(&self) -> &str {
        match self {
            FileExt::Zip => "zip",
        }
    }
}

/// Generic start print function
pub async fn start_print(
    placing_type: PlacingType,
    file_ext: FileExt,
    path: PathBuf,
) -> Result<()> {
    let url = format!(
        "{}/start/{}/{}",
        get_api_url().await,
        file_ext.to_str(),
        placing_type.to_str()
    );

    let mut form = Form::new();
    match placing_type {
        PlacingType::Local => {
            form = form.part(
                "local_file",
                Part::text(path.to_string_lossy().into_owned()),
            )
        },
        PlacingType::Remote => {
            let data = tokio::fs::read(&path)
                .await
                .map_err(|e| anyhow!("Cannot load file from this machine: {}", e))?;

            let part = Part::bytes(data).file_name(
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "file".to_owned()),
            );
            form = form.part("file", part)
        },
    };

    let response = reqwest::Client::new()
        .post(url)
        .multipart(form)
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await?;
        let message = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|json| {
                json.get("message")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| text.clone());

        error!("Cannot send model to printer: {}", message);
        anyhow::bail!("Error during printer start");
    }

    info!("Print started! Enjoy the spectacle of printing :)");
    Ok(())
}
