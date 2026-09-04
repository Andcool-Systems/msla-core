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
