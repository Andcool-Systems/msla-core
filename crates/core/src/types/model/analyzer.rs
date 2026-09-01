use crate::types::model::ir::PrintingIR;
use std::time::Duration;

#[derive(Default)]
pub struct Analyzer {
    pub current_z_pos: f64,
}

impl Analyzer {
    pub fn calc_command_duration(&mut self, ir: &PrintingIR) -> Duration {
        match ir {
            // We cannot determine the homing time precisely, so we assume it to be zero.
            PrintingIR::Home => {
                self.current_z_pos = 0.0;
                Duration::ZERO
            },

            PrintingIR::MoveZ { pos, speed } => {
                let speed_mm_s = speed / 60.0;
                let accel: f64 = 10.0;
                let distance = (pos - self.current_z_pos).abs();
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

                self.current_z_pos = *pos;
                Duration::from_secs_f64(secs)
            },

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
