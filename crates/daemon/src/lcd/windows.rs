use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

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
        debug!("Displaying layer {:?}", path);
        Ok(())
    }
}
