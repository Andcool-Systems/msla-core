use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let total_layers = 245;
    let pb = ProgressBar::new(total_layers);

    pb.set_style(
        ProgressStyle::with_template(
            "Printing [{bar:50}] {percent}%\n\
             Layer: {pos}/{len}\n\
             Time: {elapsed_precise}\n\
             ETA: {eta_precise}",
        )
        .unwrap()
        .progress_chars("=> "),
    );

    for layer in 0..total_layers {
        tokio::time::sleep(Duration::from_millis(100)).await;

        pb.set_position(layer + 1);
    }

    pb.finish_with_message("Printing finished!");
}
