use std::{path::PathBuf, time::Duration};

#[derive(Debug)]
pub enum PrintingIR {
    Home,

    MoveZ { pos: f64, speed: f64 },
    TurnUV { state: bool },

    ShowImage(PathBuf),
    Wait(Duration),

    DisableSteppers,
}
