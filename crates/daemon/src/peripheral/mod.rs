#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::PeripheralController;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::PeripheralController;

#[derive(Debug)]
pub struct PhysicalState {
    pub z_pos: f64,
    pub uv_state: bool,
}
