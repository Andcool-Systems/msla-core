use std::{path::Path, str::SplitWhitespace, time::Duration};

use crate::messaging::ir::PrintingIR;
use anyhow::{Result, anyhow};

/// Parse gcode file into PrintingIR
pub fn parse_gcode(gcode: String) -> Result<Vec<PrintingIR>> {
    let mut ir: Vec<PrintingIR> = Vec::new();

    for line in gcode.split("\n") {
        let line = line.split_once(';').map(|(left, _)| left).unwrap_or(&line);
        let mut parts = line.split_whitespace();

        let Some(command) = parts.next() else {
            continue;
        };

        match command {
            // Movement
            "G0" | "G1" => {
                let Some(com) = parse_g0(parts) else {
                    continue;
                };
                ir.push(com);
            },

            // Delay
            "G4" => ir.push(parse_g4(parts)?),

            // mm units
            "G21" => {},

            // homing
            "G28" => ir.push(PrintingIR::Home),

            // absolute pos
            "G90" => {},

            // disable steppers
            "M18" => ir.push(PrintingIR::DisableSteppers),

            // UV power
            "M106" => {
                let Some(com) = parse_m106(parts) else {
                    continue;
                };
                ir.push(com);
            },

            // Show image
            "M6054" => {
                let Some(com) = parse_m6054(parts) else {
                    continue;
                };
                ir.push(com);
            },

            gc => return Err(anyhow!("Unknown gcode: {gc}")),
        };
    }

    Ok(ir)
}

// Parse G0 G1 codes
fn parse_g0(iter: SplitWhitespace) -> Option<PrintingIR> {
    let mut z_pos: Option<f64> = None;
    let mut z_speed: Option<f64> = None;

    for part in iter {
        let mut chars = part.chars();
        let Some(letter) = chars.next() else {
            continue;
        };

        match letter {
            // Z position
            'Z' => {
                z_pos = chars.as_str().parse::<f64>().ok();
            },

            // Feedrate
            'F' => {
                z_speed = chars.as_str().parse::<f64>().ok();
            },

            _ => { // unknown parameter 
            },
        }
    }

    // If no Z-axis command is found in the current G-code, skip it
    let Some(pos) = z_pos else { return None };

    Some(PrintingIR::MoveZ {
        pos,
        speed: z_speed.unwrap_or(50.0),
    })
}

/// Parse dwell
fn parse_g4(mut iter: SplitWhitespace) -> Result<PrintingIR> {
    let mut chars = iter.next().ok_or(anyhow!("Unknown dwell"))?.chars();
    let letter = chars.next().ok_or(anyhow!("Unknown dwell: {:?}", chars))?;

    let duration = match letter {
        'P' => Duration::from_millis(chars.as_str().parse::<u64>()?),

        'S' => Duration::from_secs(chars.as_str().parse::<u64>()?),

        _ => return Err(anyhow!("Unknown dwell")),
    };

    Ok(PrintingIR::Wait(duration))
}

/// Parse uv led power
fn parse_m106(mut iter: SplitWhitespace) -> Option<PrintingIR> {
    let power = iter.find(|p| p.starts_with("S"))?;
    Some(PrintingIR::TurnUV {
        state: power.strip_prefix("S")?.parse::<f32>().ok()? > 0.0,
    })
}

/// Show image
fn parse_m6054(mut iter: SplitWhitespace) -> Option<PrintingIR> {
    let filename = iter.next()?;
    Some(PrintingIR::ShowImage(
        Path::new(&filename.replace("\"", "")).to_path_buf(),
    ))
}
