#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "windows")]
pub use win::add_to_context;

#[cfg(target_os = "windows")]
pub use win::remove_from_context;

#[cfg(not(target_os = "windows"))]
pub fn add_to_context(extensions: &[&str], label: &str, args: &str) -> ! {
    panic!("Context options not supported on this OS yet")
}

#[cfg(not(target_os = "windows"))]
pub fn remove_from_context() -> ! {
    panic!("Context options not supported on this OS yet")
}
