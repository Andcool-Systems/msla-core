use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    /// Global daemon config
    pub global: GlobalConfig,

    /// Peripheral ESP32 config
    pub peripheral: PeripheralConfig,
}

#[derive(Deserialize, Debug)]
pub struct GlobalConfig {
    /// Logging level
    pub logging_level: String,
}

#[derive(Deserialize, Debug)]
pub struct PeripheralConfig {
    pub uart: String,

    pub baud_rate: u32,
}
