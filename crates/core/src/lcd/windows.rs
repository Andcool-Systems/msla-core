use anyhow::Result;
use std::path::PathBuf;

/// Windows LCD debug
pub struct LCDController {}

impl LCDController {
    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    #[cfg(not(target_os = "linux"))]
    pub fn show_image(&self, path: PathBuf) -> Result<()> {
        println!("Displaying layer {:?}", path);
        Ok(())
    }
}
