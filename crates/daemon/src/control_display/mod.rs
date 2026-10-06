use std::thread;

use anyhow::{Result, anyhow};
use msla_core::types::printer_manager::{PrinterCommand, PrinterState};
use tokio::sync::{mpsc::Sender, watch::Receiver};
use tracing::error;

use crate::{
    control_display::{
        general_status::build_general_status_response,
        preview_loader::get_preview_bytes,
    },
    peripheral::PeripheralController,
    uart::{Uart, packet::UARTPacket},
};

mod general_status;
mod preview_loader;

/// Create display thread
pub fn spawn_control_display_thread(
    mut uart: Uart,
    sender: Sender<PrinterCommand>,
    receiver: Receiver<PrinterState>,
    peripheral: PeripheralController,
) -> Result<()> {
    thread::spawn(move || {
        loop {
            match uart.read() {
                Ok(Some(packet)) => {
                    match handle_command(
                        packet,
                        sender.clone(),
                        receiver.clone(),
                        peripheral.clone(),
                    ) {
                        Ok(Some(p)) => {
                            let _ = uart.send(p);
                        },
                        Err(e) => error!("Error at display request handle: {e}"),
                        _ => {},
                    }
                },
                Ok(None) => continue,
                Err(err) => {
                    error!("Failed to read from Uart: {:?}", err);
                    continue;
                },
            }
        }
    });

    Ok(())
}

fn handle_command(
    mut packet: UARTPacket,
    sender: Sender<PrinterCommand>,
    state: Receiver<PrinterState>,
    peripheral: PeripheralController,
) -> Result<Option<UARTPacket>> {
    Ok(match packet.packet_id {
        // General status request
        10 => Some(build_general_status_response(packet, state)),

        // Mechanical status request
        12 => {
            let mut p = UARTPacket::new_empty(packet.packet_id + 1);

            let rt = tokio::runtime::Runtime::new()?;
            let ps = rt.block_on(peripheral.get_physical_state())?;

            p.write_u8(if ps.uv_state { 1 } else { 0 });
            p.write_f32(ps.z_pos as f32);

            Some(p)
        },

        20 => {
            let side = packet.read_u16().ok_or(anyhow!("can't parse"))?;
            let count = packet.read_u16().ok_or(anyhow!("can't parse"))?;
            let offset = packet.read_u16().ok_or(anyhow!("can't parse"))?;

            let mut p = UARTPacket::new_empty(packet.packet_id + 1);

            match get_preview_bytes(state, side, count, offset) {
                Ok(result) => {
                    p.write_u8(0); // SUCCESS
                    p.write_u16(result.len() as u16);
                    p.payload.extend(result);
                },
                Err(e) => {
                    error!("Preview load error: {e}");

                    p.write_u8(1); // ERROR
                },
            };

            Some(p)
        },

        // abort command
        40 => {
            let _ = sender.blocking_send(PrinterCommand::Abort);
            None
        },

        // Home command
        56 => {
            if !state.borrow().is_busy() {
                let _ = sender.blocking_send(PrinterCommand::Home);
            }
            None
        },

        _ => None,
    })
}
