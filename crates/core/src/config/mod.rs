use crate::types::config::Config;
use anyhow::{Result, anyhow};
use std::sync::OnceLock;
use tokio::fs;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[inline]
pub async fn load(path: &str) -> Result<()> {
    let file = fs::read_to_string(path)
        .await
        .map_err(|e| anyhow!("Couldn't load config file: {}", e))?;

    let config: Config = match toml::from_str(&file) {
        Ok(config) => config,
        Err(err) => return Err(anyhow!("Couldn't parse config: {}", err.message())),
    };

    // Panics only on double-init
    let _ = CONFIG.set(config);
    Ok(())
}

#[inline]
pub async fn get_config() -> &'static Config {
    CONFIG.get().expect("Config didn't loaded")
}
