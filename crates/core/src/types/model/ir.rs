use std::{borrow::Cow, ffi::OsStr, path::PathBuf, time::Duration};

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

    pub fn to_string(&self) -> String {
        match self {
            PrintingIR::Home => "homing".to_owned(),
            PrintingIR::MoveZ(z_moving) => format!("moving to {:.2}", z_moving.pos),
            PrintingIR::TurnUV { state } => {
                format!("turning {} UV", if *state { "on" } else { "off" })
            },
            PrintingIR::ShowImage(path_buf) => format!(
                "showing image \"{}\"",
                path_buf
                    .file_name()
                    .map(OsStr::to_string_lossy)
                    .unwrap_or(Cow::Borrowed("<unknown>"))
            ),
            PrintingIR::Wait(duration) => format!("waiting {:?}", duration),
            PrintingIR::DisableSteppers => "disabling stepper".to_owned(),
            PrintingIR::EnableSteppers => "enabling stepper".to_string(),
            PrintingIR::SetStepperCurrent(c) => format!("setting stepper current to {c}mA"),
        }
    }
}

/// Timed IR contains an estimated remaining time to print finish
#[derive(Clone, Debug)]
pub struct TimedIR {
    pub ir: PrintingIR,
    pub estimated_remaining: Duration,
}

impl TimedIR {
    pub fn calc_command_duration(&self) -> Duration {
        match &self.ir {
            // We cannot determine the homing time precisely, so we assume it to be zero.
            PrintingIR::Home => Duration::ZERO,

            PrintingIR::MoveZ(m) => {
                let speed_mm_s = m.speed / 60.0;
                let accel: f64 = 10.0;
                let distance = (m.pos - m._last_pos).abs();
                let acceleration_distance = speed_mm_s.powi(2) / accel;

                let secs = if distance >= acceleration_distance {
                    // Max speed is reachable
                    let acceleration_time = 2.0 * speed_mm_s / accel;
                    let cruise_distance = distance - acceleration_distance;
                    let cruise_time = cruise_distance / speed_mm_s;

                    acceleration_time + cruise_time
                } else {
                    // Max speed is unreachable
                    let peak_speed = (distance * accel).sqrt();
                    2.0 * peak_speed / accel
                };
                Duration::from_secs_f64(secs)
            },

            PrintingIR::Wait(duration) => *duration,

            // 200ms - Approx time of communication with peripheral, awaiting answer, etc.
            PrintingIR::TurnUV { state: _ }
            | PrintingIR::EnableSteppers
            | PrintingIR::DisableSteppers
            | PrintingIR::SetStepperCurrent(_)
            | PrintingIR::ShowImage(_) => Duration::from_millis(200),
        }
    }
}
