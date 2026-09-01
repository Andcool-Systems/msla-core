use anyhow::{Result, anyhow};
use msla_core::types::model::{
    GlobalPrintingMeta,
    ir::{MetaIR, PrintingIR, TimedIR},
};
use std::{cmp::max, path::Path, str::SplitWhitespace, time::Duration};

macro_rules! try_parse_number {
    ($n:ident, $ty:ty) => {
        $n.parse::<$ty>().ok()
    };
}

pub struct GCodeParser {
    pub ir: Vec<TimedIR>,
    pub meta: GlobalPrintingMeta,
}

impl GCodeParser {
    pub fn new() -> Self {
        Self {
            ir: Vec::new(),
            meta: GlobalPrintingMeta::default(),
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
                "G28" => self.ir.push(PrintingIR::Home.to_timed_ir()),

                // absolute pos
                "G90" => {},

                // Enable steppers
                "M17" => self.ir.push(PrintingIR::EnableSteppers.to_timed_ir()),

                // disable steppers
                "M18" => self.ir.push(PrintingIR::DisableSteppers.to_timed_ir()),

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

        self.ir.push(
            PrintingIR::MoveZ {
                pos,
                speed: z_speed.unwrap_or(50.0),
            }
            .to_timed_ir(),
        )
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
            self.ir.push(PrintingIR::Wait(duration).to_timed_ir());
        }

        Ok(())
    }

    /// Parse uv led power
    fn parse_m106(&mut self, mut iter: SplitWhitespace) -> bool {
        let Some(power) = iter.find(|p| p.starts_with("S")) else {
            return false;
        };

        if let Some(power) = power
            .strip_prefix("S")
            .and_then(|power| power.parse::<f32>().ok())
        {
            self.ir
                .push(PrintingIR::TurnUV { state: power > 0.0 }.to_timed_ir());
        }

        true
    }

    /// Show image
    fn parse_m6054(&mut self, mut iter: SplitWhitespace) -> bool {
        let Some(filename) = iter.next() else {
            return false;
        };
        self.ir.push(
            PrintingIR::ShowImage(Path::new(&filename.trim_matches('"')).to_path_buf())
                .to_timed_ir(),
        );

        true
    }

    /// Parse metadata from comments
    fn parse_comment(&mut self, line: &str) -> bool {
        let (comm, data) = line.split_once(':').unwrap_or((line, ""));

        match comm.trim().to_lowercase().as_str() {
            "layer_start" => {
                let Some(n) = try_parse_number!(data, usize) else {
                    return false;
                };
                let ex = self.meta.total_layer_count.unwrap_or(0);
                self.meta.total_layer_count = Some(max(ex, n + 1));

                self.ir
                    .push(PrintingIR::Meta(MetaIR::LayerStart(n)).to_timed_ir());
            },

            "layer_end" => {
                self.ir
                    .push(PrintingIR::Meta(MetaIR::LayerEnd).to_timed_ir());
            },

            x => {
                self.parse_metadata(x, data);
            },
        }

        true
    }

    /// Try parse meta from gcode file
    fn parse_metadata(&mut self, key: &str, data: &str) -> bool {
        match key {
            "estimatedprinttime" => {
                self.meta.estimated_printing_time = try_parse_number!(data, usize);
            },

            "volume" => {
                self.meta.volume = try_parse_number!(data, f32);
            },

            "filename" => {
                self.meta.file_name = Some(data.to_owned());
            },

            "weight" => {
                self.meta.weight = try_parse_number!(data, f32);
            },

            "price" => {
                self.meta.price = try_parse_number!(data, f32);
            },

            "layerheight" => {
                self.meta.layer_height = try_parse_number!(data, f32);
            },

            "totallayer" => {
                let ex = self.meta.total_layer_count.unwrap_or(0);

                if let Some(n) = try_parse_number!(data, usize) {
                    self.meta.total_layer_count = Some(max(ex, n));
                }
            },

            _ => {},
        }

        true
    }
}
