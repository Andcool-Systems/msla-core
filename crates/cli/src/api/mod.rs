use std::path::PathBuf;

use anyhow::{Result, anyhow};
use msla_core::types::cli::api::status::StatusResponse;
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use serde_json::Value;
use tracing::info;

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
    pub url: String,
}

impl ApiService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            client: Client::new(),
            url: format!("http://{}:{}", host, port),
        }
    }

    /// Extract json value by key
    fn extract_field(json: &String, key: &str) -> Option<String> {
        serde_json::from_str::<Value>(json)
            .ok()
            .and_then(|json| json.get(key).and_then(Value::as_str).map(str::to_owned))
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
            let text = response.text().await?;
            anyhow::bail!(
                "Cannot fetch current status: {}",
                Self::extract_field(&text, "message").unwrap_or(text)
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
            anyhow::bail!(
                "Cannot send model to printer: {}",
                Self::extract_field(&text, "message").unwrap_or(text)
            );
        }

        info!("Print started! Enjoy the spectacle of printing :)");
        Ok(())
    }

    /// Abort current print
    pub async fn abort(&self) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/abort", self.url))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            anyhow::bail!(
                "Cannot abort printing: {}",
                Self::extract_field(&text, "message").unwrap_or(text)
            );
        }

        info!("Print aborted");
        Ok(())
    }

    /// Home Z axis
    pub async fn home(&self) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/home", self.url))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            anyhow::bail!(
                "Cannot send home signal to printer: {}",
                Self::extract_field(&text, "message").unwrap_or(text)
            );
        }

        info!("Print homing...");
        Ok(())
    }

    /// Disable Z stepper
    pub async fn disable_stepper(&self) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/disable-stepper", self.url))
            .send()
            .await?;

        if !response.status().is_success() {
            let text = response.text().await?;
            anyhow::bail!(
                "Cannot send disable stepper signal to printer: {}",
                Self::extract_field(&text, "message").unwrap_or(text)
            );
        }

        info!("Z stepper disabled");
        Ok(())
    }
}
