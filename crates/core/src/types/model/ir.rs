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
    /// Convert to timed ir
    pub fn to_timed_ir(&self) -> TimedIR {
        TimedIR {
            ir: self.clone(),
            estimated_remaining: Duration::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetaIR {
    LayerStart(usize),
    LayerEnd,
}

/// Timed IR contains an estimated remaining time to print finish
#[derive(Clone, Debug)]
pub struct TimedIR {
    pub ir: PrintingIR,
    pub estimated_remaining: Duration,
}
