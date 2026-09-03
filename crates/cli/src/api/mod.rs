use std::path::PathBuf;

use anyhow::Result;
use msla_core::{config, types::cli::api::status::StatusResponse};
use reqwest::Client;
use serde_json::json;

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
    let response = match reqwest::get(format!("{}/status", get_api_url().await)).await {
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

pub async fn start_local_print(path: PathBuf) -> Result<()> {
    let client = Client::new();

    let response = client
        .post(format!("{}/start/zip/local", get_api_url().await))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_vec(&json!({"path": path}))?)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!(
            "Cannot start print: ({}) {}",
            response.status().as_u16(),
            response.text().await?
        )
    }
    Ok(())
}
