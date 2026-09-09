use std::{
    io::ErrorKind,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use anyhow::{Result, anyhow};
use msla_core::types::cli::dyn_config::{Connectivity, DynConfig};
use tokio::{fs, sync::RwLock};

static DYN_CONFIG: OnceLock<Arc<RwLock<DynConfig>>> = OnceLock::new();

/// Get path of dynamic user config
fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("OpenMSLA")
        .join("config.toml")
}

/// Acquires config
pub fn config() -> Result<&'static Arc<RwLock<DynConfig>>> {
    DYN_CONFIG.get().ok_or(anyhow!("Config not loaded"))
}

/// Save config into file
pub async fn write_config(conf: &DynConfig) -> Result<()> {
    let path = get_config_path();
    fs::create_dir_all(path.parent().unwrap()).await?;
    fs::write(path, toml::to_string_pretty(conf)?).await?;

    Ok(())
}

/// Inits the config
pub async fn init_config() -> Result<()> {
    let config = match fs::read_to_string(get_config_path()).await {
        Ok(content) => {
            let value: toml::Value = toml::from_str(&content)?;
            let config: DynConfig = value.clone().try_into()?;

            let normalized = toml::to_string_pretty(&config)?;
            let original = toml::to_string_pretty(&value)?;

            if normalized != original {
                write_config(&config).await?;
            }

            config
        },

        Err(e) if e.kind() == ErrorKind::NotFound => {
            let config = DynConfig {
                connectivity: Connectivity {
                    alt_default: false,
                    default_port: 709,
                    default_scan_port: 710,
                },
            };

            write_config(&config).await?;
            config
        },

        Err(e) => return Err(e.into()),
    };

    DYN_CONFIG
        .set(Arc::new(RwLock::new(config)))
        .map_err(|_| anyhow!("Config already initialized"))
}

#[macro_export]
macro_rules! modify_config {
   ($($field:ident).+ = $value:expr) => {{
        let mut config = $crate::dyn_config::config()?.write().await;
        config.$($field).+ = $value;
    }};
}
