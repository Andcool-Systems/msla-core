use std::thread;

use anyhow::{Result, anyhow};
use msla_core::types::printer_manager::{PrinterCommand, PrinterState};
use tokio::sync::{mpsc::Sender, watch::Receiver};
use tracing::error;

use crate::{
    control_display::preview_loader::get_preview_bytes,
    peripheral::PeripheralController,
    uart::{Uart, packet::UARTPacket},
};

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
        10 => {
            let mut p = UARTPacket::new_empty(packet.packet_id + 1);

            match state.borrow().clone() {
                PrinterState::Idle => p.write_u8(0),
                PrinterState::Printing(s) => {
                    let current_ir = s.model.ir.get(s.current_ir_index);
                    let est = current_ir
                        .map(|ir| ir.estimated_remaining)
                        .unwrap_or_default();

                    let current_ir_duration = current_ir
                        .map(|ir| ir.calc_command_duration())
                        .unwrap_or_default();

                    p.write_u8(1);

                    // LAYERS
                    // current layer
                    p.write_u16(s.printing_layer as u16);
                    // total layers
                    p.write_u16(s.model.model_meta.total_layer_count as u16);

                    // IR
                    // current ir index
                    p.write_u32(s.current_ir_index as u32);
                    // total ir len
                    p.write_u32(s.model.ir.len() as u32);

                    // IR DURATION
                    // duration of current ir
                    p.write_u32(current_ir_duration.as_secs_f32() as u32);
                    // full estimated printing time
                    p.write_u32(s.model.model_meta.estimated_printing_time as u32);
                    // estimated finish time
                    p.write_u32(
                        est.checked_sub(s.current_ir_elapsed.elapsed())
                            .unwrap_or_default()
                            .as_secs() as u32,
                    );

                    // total elapsed secs
                    p.write_u32(s.total_elapsed.elapsed().as_secs() as u32);
                },
                PrinterState::Paused(_) => p.write_u8(2),
                PrinterState::Error(_) => p.write_u8(3),
                PrinterState::Busy => p.write_u8(4),
                PrinterState::Aborted => p.write_u8(5),
                PrinterState::Finished { total_elapsed: _ } => p.write_u8(6),
            };

            Some(p)
        },

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

            Some(match get_preview_bytes(state, side, count, offset) {
                Ok(result) => {
                    let mut p = UARTPacket::new_empty(packet.packet_id + 1);
                    p.write_u8(0); // SUCCESS
                    p.write_u16(result.len() as u16);
                    p.payload.extend(result);

                    p
                },
                Err(e) => {
                    error!("Preview load error: {e}");
                    let mut p = UARTPacket::new_empty(packet.packet_id + 1);
                    p.write_u8(1); // ERROR

                    p
                },
            })
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
