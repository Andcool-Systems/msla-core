use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

/// Windows LCD debug
#[derive(Clone)]
pub struct LCDController {}

impl LCDController {
    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    #[cfg(not(target_os = "linux"))]
    pub fn show_image(&self, path: PathBuf) -> Result<()> {
        info!("Displaying layer {:?}", path);
        Ok(())
    }
}
