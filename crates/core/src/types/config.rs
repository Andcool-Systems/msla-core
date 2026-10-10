use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Global daemon config
    pub global: GlobalConfig,

    /// Peripheral ESP32 config
    pub peripheral: PeripheralConfig,

    /// REST API config
    pub rest_api: REST,

    /// Broadcast listener
    pub broadcast_listener: BroadcastListener,

    /// Physical machine parameters
    pub physical: Physical,

    /// Control display params
    pub control_display: ControlDisplay,

    /// Auto-mounter of usb device
    pub usb_controller: UsbMounter,
}

#[derive(Deserialize, Debug)]
pub struct GlobalConfig {
    /// Logging level
    pub logging_level: String,

    /// Printer name
    pub machine_name: String,
}

#[derive(Deserialize, Debug)]
pub struct PeripheralConfig {
    pub uart: String,

    pub baud_rate: u32,
}

#[derive(Deserialize, Debug)]
pub struct REST {
    /// Server address
    pub addr: String,

    /// Server port
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct BroadcastListener {
    /// bc address
    pub addr: String,

    /// bc port
    pub port: u16,
}

#[derive(Deserialize, Debug)]
pub struct Physical {
    /// Max machine height (mm)
    pub machine_height: f32,
}

#[derive(Deserialize, Debug)]
pub struct ControlDisplay {
    pub uart: String,

    pub baud_rate: u32,
}

#[derive(Deserialize, Debug)]
pub struct UsbMounter {
    /// Path for scanning a usb devices, e.g. /dev/disk/by-path
    pub scan_path: PathBuf,

    /// Regex for specific usb device/port
    pub usb_path_regex: String,

    /// Path for mounting root (/mnt/usb)
    pub mount_point: PathBuf,

    /// Period of dir scanning (seconds)
    pub scan_period: u32,
}
