use anyhow::Result;
use msla_core::{config, types::cli::api::status::StatusResponse};

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
