use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum MetaIR {
    LayerStart(u64),
    LayerEnd,
}
