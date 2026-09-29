use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

/// Windows LCD debug
#[derive(Clone)]
pub struct LCDController {}

impl LCDController {
    pub async fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn show_image(&self, path: PathBuf) -> Result<()> {
        debug!("Displaying layer {:?}", path);
        Ok(())
    }
}
