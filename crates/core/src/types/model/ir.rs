use std::{path::PathBuf, time::Duration};

#[derive(Clone, Debug, PartialEq)]
pub enum PrintingIR {
    Home,

    MoveZ(ZMoving),
    TurnUV { state: bool },

    ShowImage(PathBuf),
    Wait(Duration),

    DisableSteppers,
    EnableSteppers,
    SetStepperCurrent(u16),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ZMoving {
    pub pos: f64,
    pub speed: f64,
    pub _last_pos: f64,
}

impl ZMoving {
    pub fn new(pos: f64, speed: f64) -> Self {
        Self {
            pos,
            speed,
            _last_pos: 0.0,
        }
    }
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

/// Timed IR contains an estimated remaining time to print finish
#[derive(Clone, Debug)]
pub struct TimedIR {
    pub ir: PrintingIR,
    pub estimated_remaining: Duration,
}
