use std::{path::PathBuf, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum PrintingIR {
    Home,

    MoveZ { pos: f64, speed: f64 },
    TurnUV { state: bool },

    ShowImage(PathBuf),
    Wait(Duration),

    DisableSteppers,
    EnableSteppers,

    Meta(MetaIR),
}

impl PrintingIR {
    /// Calculate the approximate execution time of an IR command
    pub fn get_approx_duration(&self) -> Duration {
        match self {
            // We cannot determine the homing time precisely, so we assume it to be zero.
            PrintingIR::Home => Duration::ZERO,

            PrintingIR::MoveZ { pos, speed } => Duration::from_secs_f64(pos / (speed / 60.0)),

            PrintingIR::Wait(duration) => *duration,

            // 500ms - Approx time of communication with peripheral, awaiting answer, etc.
            PrintingIR::TurnUV { state: _ }
            | PrintingIR::EnableSteppers
            | PrintingIR::DisableSteppers
            | PrintingIR::ShowImage(_) => Duration::from_millis(500),

            PrintingIR::Meta(_) => Duration::ZERO,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetaIR {
    LayerStart(usize),
    LayerEnd,
}
