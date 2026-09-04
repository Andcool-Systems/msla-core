use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use msla_core::types::cli::api::status::StatusResponse;
use tokio::time::sleep;
use tracing::{error, info};

use crate::api::ApiService;

fn format_duration(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    match (hours, minutes) {
        (h, _) if h > 0 => format!("{h}h {minutes}m {seconds}s"),
        (_, m) if m > 0 => format!("{m}m {seconds}s"),
        _ => format!("{seconds}s"),
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Display cli statusbar
pub async fn show_status(api_client: &ApiService, watch: bool, period: u64) -> Result<()> {
    let pb = ProgressBar::new(100);
    let mut instant = Instant::now();
    let duration = Duration::from_secs(period);
    let mut status: StatusResponse = api_client.get_status().await?;
    let mut estimated = 0f64;
    let mut updated = false;

    pb.set_style(
        ProgressStyle::with_template(&format!("{} [{{bar:40}}] {{msg}}", "Printing".green()))
            .unwrap()
            .progress_chars("=> "),
    );

    loop {
        if instant.elapsed() > duration {
            status = api_client.get_status().await?;
            updated = true;
            instant = Instant::now();
        }

        match status.state.as_str() {
            "printing" | "paused" => {
                let current_status = status
                    .current_status
                    .as_ref()
                    .ok_or(anyhow!("Can't find current status in json response"))?;

                let model_meta = status
                    .model_meta
                    .as_ref()
                    .ok_or(anyhow!("Can't find model meta in json response"))?;

                if updated {
                    pb.set_style(
                        ProgressStyle::with_template(&format!(
                            "{} [{{bar:40}}] {{msg}}",
                            capitalize(&status.state).green()
                        ))
                        .unwrap()
                        .progress_chars("=> "),
                    );
                    estimated = current_status.estimated_finish_time;
                }

                let percent =
                    (current_status.current_ir_index as f64 / model_meta.ir_len as f64) * 100.0;

                pb.set_position(percent.round() as u64);

                let mut message_lines: Vec<String> = Vec::new();
                message_lines.push(format!("{percent:.2}%"));
                message_lines.push(format!(
                    "{}: {}/{}",
                    "Layer".bold(),
                    current_status.current_layer,
                    model_meta.total_layer_count
                ));
                message_lines.push(format!(
                    "{}: {:.2}/{:.2}mm",
                    "Height".bold(),
                    current_status.current_layer as f64 * 0.05, // TODO: FIXME: REPLACE TO status.model_meta.layer_height
                    model_meta.total_layer_count as f64 * 0.05
                ));
                message_lines.push(format!(
                    "{}: {}",
                    "ETA".bold(),
                    format_duration(estimated as u64)
                ));

                if !watch {
                    pb.finish_with_message(message_lines.join("\n"));
                    break;
                }

                pb.set_message(message_lines.join("\n"));

                sleep(Duration::from_secs(1)).await;
                estimated = (estimated - 1.0).max(0.0);
            },
            "aborted" => {
                pb.finish_with_message("Printing aborted");
                return Ok(());
            },
            "error" => {
                let e = status.error.unwrap_or("Unknown printer error".to_owned());
                error!("Error during printing: {}", e);
                anyhow::bail!(e)
            },
            "idle" => {
                pb.finish_with_message("Printer is idle");
                return Ok(());
            },
            "finished" => {
                pb.finish_with_message("Printing finished, Goodbye <3");
                return Ok(());
            },
            s => {
                info!("Printer is {}", s);
                return Ok(());
            },
        }

        updated = false;
    }

    Ok(())
}
