use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub peripheral: PeripheralConfig,
}

#[derive(Deserialize, Debug)]
pub struct PeripheralConfig {
    pub uart: String,

    pub baud_rate: u32,
}
