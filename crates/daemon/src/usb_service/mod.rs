use anyhow::Result;
use msla_core::config;
use regex::Regex;
use std::{collections::HashSet, path::PathBuf};
use tokio::fs;

/// Gets all device list, matching provided regex
async fn get_device_list(path: &PathBuf, re: &Regex) -> Result<HashSet<PathBuf>> {
    if !path.exists() {
        anyhow::bail!("Path {:?} not exists", path);
    }

    if !path.is_dir() {
        anyhow::bail!("Path {:?} not a directory", path);
    }

    let mut found_hashmap = HashSet::new();
    let mut read_dir = fs::read_dir(&path).await?;
    while let Some(file) = read_dir.next_entry().await? {
        let name = file.file_name().to_string_lossy().into_owned();

        if re.is_match(&name) {
            found_hashmap.insert(file.path().canonicalize()?);
        }
    }

    Ok(found_hashmap)
}

/// Starts usb mounter service loop
pub async fn start_usb_service() -> Result<()> {
    let config = config::get_config().await;
    let re = Regex::new(&config.usb_controller.usb_path_regex)?;

    println!(
        "S: {:?}",
        get_device_list(&config.usb_controller.scan_path, &re).await,
    );

    Ok(())
}
