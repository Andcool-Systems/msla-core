use crate::types::model::ir::PrintingIR;
use std::time::Duration;

#[derive(Default)]
pub struct Analyzer {}

impl Analyzer {
    pub fn calc_command_duration(&mut self, ir: &PrintingIR) -> Duration {
        match ir {
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

            // 500ms - Approx time of communication with peripheral, awaiting answer, etc.
            PrintingIR::TurnUV { state: _ }
            | PrintingIR::EnableSteppers
            | PrintingIR::DisableSteppers
            | PrintingIR::SetStepperCurrent(_)
            | PrintingIR::ShowImage(_) => Duration::from_millis(200),
        }
    }
}
