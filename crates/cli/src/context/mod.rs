#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "windows")]
pub use win::add_to_context;

#[cfg(target_os = "windows")]
pub use win::remove_from_context;

#[cfg(not(target_os = "windows"))]
use anyhow::Result;

#[cfg(not(target_os = "windows"))]
pub async fn add_to_context(label: &str) -> Result<()> {
    panic!("Context options not supported on this OS yet")
}

#[cfg(not(target_os = "windows"))]
pub async fn remove_from_context() -> Result<()> {
    panic!("Context options not supported on this OS yet")
}
