use crate::{
    api::{FileExt, PlacingType, start_print},
    search::execute_search,
    status::show_status,
};
use anyhow::Result;
use clap::Parser;
use msla_core::{
    config, logging,
    types::cli::args::{Args, Command},
};
use std::path::PathBuf;
use tracing::error;
mod api;
mod search;
mod status;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logger(tracing::Level::INFO, false);
    config::load("config.toml").await.map_err(|e| {
        error!("{}", e);
        std::process::exit(-1);
    });
    let args = Args::parse();

    match args.command {
        Command::Start(start_args) => {
            let ext = if start_args.zip {
                FileExt::Zip
            } else {
                error!("Please, specify the file type: --zip or others");
                return Ok(());
            };

            if let Some(local) = start_args.local {
                start_print(PlacingType::Local, ext, PathBuf::from(local)).await?;
            } else if let Some(remote) = start_args.remote {
                start_print(PlacingType::Remote, ext, PathBuf::from(remote)).await?;
            } else {
                error!("Please, specify the file path: --local <path> or --remote <path>")
            }

            show_status(true, 10).await?;
        },
        Command::Abort => todo!(),
        Command::Pause => todo!(),
        Command::Resume => todo!(),
        Command::Status(status_args) => {
            show_status(status_args.watch, status_args.period.unwrap_or(10)).await?;
        },
        Command::Search(search_args) => {
            execute_search(search_args.interval.unwrap_or(2), search_args.alt).await?;
        },
    }

    Ok(())
}
