#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use crate::lcd::linux::LCDController;

#[cfg(not(target_os = "linux"))]
pub mod windows;

#[cfg(not(target_os = "linux"))]
pub use crate::lcd::windows::LCDController;
