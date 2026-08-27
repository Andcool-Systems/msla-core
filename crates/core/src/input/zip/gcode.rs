use std::{path::Path, str::SplitWhitespace, time::Duration};

use crate::{
    messaging::ir::{MetaIR, PrintingIR},
    types::ir::GlobalPrintingMeta,
};
use anyhow::{Result, anyhow};

pub struct GCodeParser {
    pub ir: Vec<PrintingIR>,
    pub meta: GlobalPrintingMeta,
}

impl GCodeParser {
    pub fn new() -> Self {
        Self {
            ir: Vec::new(),
            meta: GlobalPrintingMeta::new(),
        }
    }

    /// Parse gcode file into PrintingIR
    pub fn parse_gcode(&mut self, gcode: String) -> Result<()> {
        for line in gcode.lines() {
            let (code, comment) = line.split_once(';').unwrap_or((line, ""));

            // Parse comment (probably meta)
            if !comment.is_empty() {
                self.parse_comment(comment);
            }

            let mut parts = code.split_whitespace();

            let Some(command) = parts.next() else {
                continue;
            };

            match command {
                // Movement
                "G0" | "G1" => self.parse_g0(parts),

                // Delay
                "G4" => self.parse_g4(parts)?,

                // mm units
                "G21" => {},

                // homing
                "G28" => self.ir.push(PrintingIR::Home),

                // absolute pos
                "G90" => {},

                // disable steppers
                "M18" => self.ir.push(PrintingIR::DisableSteppers),

                // UV power
                "M106" => {
                    self.parse_m106(parts);
                },

                // Show image
                "M6054" => {
                    self.parse_m6054(parts);
                },

                gc => return Err(anyhow!("Unknown gcode: {gc}")),
            };
        }

        Ok(())
    }

    // Parse G0 G1 codes
    fn parse_g0(&mut self, iter: SplitWhitespace) {
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
        let Some(pos) = z_pos else { return };

        self.ir.push(PrintingIR::MoveZ {
            pos,
            speed: z_speed.unwrap_or(50.0),
        })
    }

    /// Parse dwell
    fn parse_g4(&mut self, mut iter: SplitWhitespace) -> Result<()> {
        let mut chars = iter.next().ok_or(anyhow!("Unknown dwell"))?.chars();
        let letter = chars.next().ok_or(anyhow!("Unknown dwell: {:?}", chars))?;

        let duration = match letter {
            'P' => Duration::from_millis(chars.as_str().parse::<u64>()?),

            'S' => Duration::from_secs(chars.as_str().parse::<u64>()?),

            _ => return Err(anyhow!("Unknown dwell")),
        };

        if !duration.is_zero() {
            self.ir.push(PrintingIR::Wait(duration));
        }

        Ok(())
    }

    /// Parse uv led power
    fn parse_m106(&mut self, mut iter: SplitWhitespace) -> Option<()> {
        let power = iter.find(|p| p.starts_with("S"))?;
        self.ir.push(PrintingIR::TurnUV {
            state: power.strip_prefix("S")?.parse::<f32>().ok()? > 0.0,
        });

        Some(())
    }

    /// Show image
    fn parse_m6054(&mut self, mut iter: SplitWhitespace) -> Option<()> {
        let filename = iter.next()?;
        self.ir.push(PrintingIR::ShowImage(
            Path::new(&filename.trim_matches('"')).to_path_buf(),
        ));

        Some(())
    }

    /// Parse metadata from comments
    fn parse_comment(&mut self, line: &str) {
        let Some(res) = (match line.to_lowercase() {
            l if l.starts_with("layer_start") => {
                let l = l
                    .replace("layer_start:", "")
                    .parse::<u32>()
                    .ok()
                    .map(|n| PrintingIR::Meta(MetaIR::LayerStart(n)));

                if let Some(PrintingIR::Meta(MetaIR::LayerStart(n))) = l {
                    self.meta.total_layer_count = n;
                }

                l
            },

            l if l.starts_with("layer_end") => Some(PrintingIR::Meta(MetaIR::LayerEnd)),

            _ => None,
        }) else {
            return;
        };

        self.ir.push(res);
    }
}
