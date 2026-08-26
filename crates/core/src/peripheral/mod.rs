use std::{sync::Arc, time::Duration};

use crate::{
    config::get_config,
    uart::{UART, packet::UARTPacket, uart_client::UARTClient},
};
use anyhow::{Result, anyhow};

pub enum MovingZStatus {
    Success,
    Unknown,
}

pub struct PeripheralController {
    uart: Arc<UARTClient>,
}

impl PeripheralController {
    /// Creates new peripheral controller
    pub async fn new() -> Result<Self> {
        let config = get_config().await;

        Ok(Self {
            uart: UARTClient::new(UART::open(
                config.peripheral.uart.clone(),
                config.peripheral.baud_rate,
            )?)?,
        })
    }

    /// Home Z axis
    pub async fn home_z(&self) -> Result<UARTPacket> {
        self.uart
            .request(UARTPacket::new(56, &[]), Duration::from_secs(150), 3)
            .await
    }

    /// Moves Z axis to a provided pos and with provided speed
    pub async fn move_z_to(&self, pos: f64, speed: f64) -> Result<MovingZStatus> {
        let mut buf: Vec<u8> = Vec::new();

        buf.extend(pos.to_le_bytes());
        buf.extend(speed.to_le_bytes());

        let mut packet = self
            .uart
            .request(
                UARTPacket::new(50, &buf),
                Duration::from_secs_f64((pos / (speed / 60.0)) + 30.0),
                3,
            )
            .await?;

        let status = match packet.read_u8() {
            Some(0) => MovingZStatus::Success,
            _ => MovingZStatus::Unknown,
        };

        Ok(status)
    }

    /// Turn UV backlight
    pub async fn turn_uv(&self, state: bool) -> Result<()> {
        self.uart
            .request(
                UARTPacket::new(40, &[state.into()]),
                Duration::from_millis(500),
                3,
            )
            .await?;

        Ok(())
    }

    /// Sets a motor current and returns a current current
    pub async fn set_motor_current(&self, current: u16) -> Result<u16> {
        let mut packet = UARTPacket::new_empty(52);
        packet.write_u16(current);

        let mut packet = self
            .uart
            .request(packet, Duration::from_millis(500), 3)
            .await?;

        packet
            .read_u16()
            .ok_or(anyhow!("Current in packet not found"))
    }

    /// Disable stepper (driver EN pin)
    pub async fn disable_steppers(&self) -> Result<()> {
        self.uart
            .request(UARTPacket::new(58, &[]), Duration::from_millis(500), 3)
            .await?;

        Ok(())
    }

    /// Enable stepper (driver EN pin)
    pub async fn enable_steppers(&self) -> Result<()> {
        self.uart
            .request(UARTPacket::new(60, &[]), Duration::from_millis(500), 3)
            .await?;

        Ok(())
    }
}

impl Drop for PeripheralController {
    fn drop(&mut self) {
        let _ = self.turn_uv(false);
    }
}
