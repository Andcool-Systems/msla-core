use std::time::{Duration, Instant};

use crate::api::ApiService;
use anyhow::{Result, anyhow};
use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use msla_core::types::cli::api::status::StatusResponse;
use tokio::time::sleep;
use tokio_retry::{
    Retry,
    strategy::{ExponentialBackoff, jitter},
};
use tracing::{error, info};

pub fn format_duration(dur: Duration) -> String {
    let total_seconds = dur.as_secs();
    let days = total_seconds / (3600 * 24);
    let hours = (total_seconds % (3600 * 24)) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    match (days, hours, minutes) {
        (d, _, _) if d > 0 => format!("{d}d {hours}h {minutes}m {seconds}s"),
        (_, h, _) if h > 0 => format!("{h}h {minutes}m {seconds}s"),
        (_, _, m) if m > 0 => format!("{m}m {seconds}s"),
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
    let mp = MultiProgress::new();
    let pb_total = mp.add(ProgressBar::new(100));
    let pb_it_ref = ProgressBar::new(100);

    let mut instant = Instant::now();
    let duration = Duration::from_secs(period);
    let mut status: StatusResponse = api_client.get_status().await?;
    let mut estimated = Duration::ZERO;
    let mut estimated_elapsed = Instant::now();
    let mut current_ir_instant = Instant::now();
    let mut updated = true;
    let mut first_init = true;
    let mut fetch_flag = false;

    pb_total.set_style(
        ProgressStyle::with_template(&format!("{} [{{bar:40}}] {{msg}}", "Printer".green()))
            .unwrap()
            .progress_chars("=> "),
    );

    let retry_strategy = ExponentialBackoff::from_millis(100).map(jitter).take(3);
    loop {
        if instant.elapsed() > duration || fetch_flag {
            status = Retry::start(retry_strategy.clone(), || async {
                api_client.get_status().await.map_err(|e| {
                    error!("{}", e);
                    e
                })
            })
            .await?;
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
                    let mut header = capitalize(&status.state);
                    header.push_str("...");

                    let mut ir_header_str = capitalize(&current_status.current_ir_description);
                    ir_header_str.push_str("...");
                    let header_max_len = header.len().max(ir_header_str.len());

                    pb_total.set_style(
                        ProgressStyle::with_template(&format!(
                            "{}{} [{{bar:40}}] {{msg}}",
                            header.green(),
                            " ".repeat(header_max_len.saturating_sub(header.len()))
                        ))
                        .unwrap()
                        .progress_chars("=> "),
                    );

                    pb_it_ref.set_style(
                        ProgressStyle::with_template(&format!(
                            "{}{} [{{bar:40}}] {{msg}}",
                            ir_header_str.cyan(),
                            " ".repeat(header_max_len.saturating_sub(ir_header_str.len()))
                        ))
                        .unwrap()
                        .progress_chars("=> "),
                    );

                    if first_init {
                        mp.add(pb_it_ref.clone());
                        first_init = false;
                    }

                    estimated = Duration::from_secs_f64(current_status.estimated_finish_time);
                    estimated_elapsed = Instant::now();
                    current_ir_instant = Instant::now();
                }

                let percent =
                    (current_status.current_ir_index as f64 / model_meta.ir_len as f64) * 100.0;

                pb_total.set_position(percent.round() as u64);
                pb_total.set_message(format!(
                    "{percent:.2}% ({}: {})",
                    "ETA".bold(),
                    format_duration(estimated.saturating_sub(estimated_elapsed.elapsed()))
                ));

                let mut elapsed =
                    current_ir_instant.elapsed().as_secs_f64() + current_status.current_ir_elapsed;

                if elapsed > current_status.current_ir_duration
                    && current_status.current_ir_duration != 0.0
                {
                    elapsed = current_status.current_ir_duration;
                    fetch_flag = true;
                }

                let ir_elapsed_percent = (elapsed / current_status.current_ir_duration) * 100.0;
                pb_it_ref.set_position(ir_elapsed_percent.round() as u64);

                let mut message_lines: Vec<String> = Vec::new();
                message_lines.push(format!(
                    "{:.2}/{:.2}s",
                    elapsed, current_status.current_ir_duration
                ));
                message_lines.push(format!(
                    "{}: {}/{}",
                    "Layer".bold(),
                    current_status.current_layer,
                    model_meta.total_layer_count
                ));
                message_lines.push(format!(
                    "{}: {}/{}",
                    "IR".bold(),
                    current_status.current_ir_index,
                    model_meta.ir_len
                ));
                message_lines.push(format!(
                    "{}: {:.2}/{:.2}mm",
                    "Height".bold(),
                    current_status.current_layer as f64 * model_meta.layer_height.unwrap_or(0.05),
                    model_meta.total_layer_count as f64 * model_meta.layer_height.unwrap_or(0.05)
                ));
                message_lines.push(format!(
                    "{}: {}",
                    "Total elapsed".bold(),
                    format_duration(
                        Duration::from_secs_f64(current_status.total_elapsed)
                            + estimated_elapsed.elapsed()
                    )
                ));

                pb_it_ref.set_message(message_lines.join("\n"));

                if !watch {
                    pb_total.abandon();
                    pb_it_ref.abandon();
                    break;
                }

                sleep(Duration::from_millis(1000)).await;
            },
            s => {
                mp.remove(&pb_it_ref);

                match s {
                    "aborted" => {
                        pb_total.finish_with_message("Printing aborted");
                        return Ok(());
                    },
                    "error" => {
                        let e = status.error.unwrap_or("Unknown printer error".to_owned());
                        error!("Error during printing: {}", e);
                        anyhow::bail!(e)
                    },
                    "idle" => {
                        pb_total.finish_with_message("Printer is idle");
                        return Ok(());
                    },
                    "finished" => {
                        pb_total.finish_with_message("Printing finished, Goodbye <3");
                        return Ok(());
                    },
                    "busy" => {
                        pb_total.finish_with_message("Printer is busy");
                        return Ok(());
                    },
                    s => {
                        info!("Printer is {}", s);
                        return Ok(());
                    },
                }
            },
        }

        updated = false;
    }

    Ok(())
}
