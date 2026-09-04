use std::path::PathBuf;

use anyhow::{Result, anyhow};
use msla_core::types::cli::api::status::StatusResponse;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use tracing::{error, info};

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

pub struct ApiService {
    client: Client,
    url: String,
}

impl ApiService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: Client::new(),
            url: format!("http://{}:{}", host, port),
        }
    }

    /// Get current printing status
    pub async fn get_status(&self) -> Result<StatusResponse> {
        let response = match self.client.get(format!("{}/status", self.url)).send().await {
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

    /// Generic start print function
    pub async fn start_print(
        &self,
        placing_type: PlacingType,
        file_ext: FileExt,
        path: PathBuf,
    ) -> Result<()> {
        let url = format!(
            "{}/start/{}/{}",
            self.url,
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

        let response = self.client.post(url).multipart(form).send().await?;

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
}
