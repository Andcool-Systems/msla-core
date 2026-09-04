use crate::{
    api::{ApiService, FileExt, PlacingType},
    search::execute_search,
    status::show_status,
};
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use msla_core::{
    config::{self, get_config},
    logging,
    types::cli::args::{Args, Command},
};
use std::{net::IpAddr, path::PathBuf};
use tracing::error;
mod api;
mod search;
mod status;

async fn get_printers(alt_scan: bool) -> Result<IpAddr> {
    let found = execute_search(1, alt_scan).await?;

    if found.is_empty() {
        return Err(anyhow!(
            "Printers in this local network not found. Specify IP `--host <ip>`"
        ));
    }

    if found.len() == 1 {
        return Ok(found.first().unwrap().ip);
    }

    println!("{}", format!("{} printers found:", found.len()).bold());

    let options = found
        .iter()
        .map(|p| {
            format!(
                "Printer \"{}\", ver {} ({})",
                p.name.as_ref().unwrap_or(&"<unknown>".to_string()),
                p.ver.as_ref().unwrap_or(&"<unknown>".to_string()),
                p.ip
            )
        })
        .collect::<Vec<String>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select option")
        .items(options)
        .default(0)
        .interact()?;

    Ok(found.get(selection).unwrap().ip)
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logger(tracing::Level::INFO, false);
    config::load("config.toml").await.map_err(|e| {
        error!("{}", e);
        std::process::exit(-1);
    });
    let config = get_config().await;
    let args = Args::parse();

    let api_client = ApiService::new(
        args.host
            .unwrap_or(get_printers(args.alt_scan).await?.to_string()),
        args.port.unwrap_or(config.rest_api.port),
    );

    match args.command {
        Command::Start(start_args) => {
            let ext = if start_args.zip {
                FileExt::Zip
            } else {
                error!("Please, specify the file type: --zip or others");
                return Ok(());
            };

            if let Some(local) = start_args.local {
                api_client
                    .start_print(PlacingType::Local, ext, PathBuf::from(local))
                    .await?;
            } else if let Some(remote) = start_args.remote {
                api_client
                    .start_print(PlacingType::Remote, ext, PathBuf::from(remote))
                    .await?;
            } else {
                error!("Please, specify the file path: --local <path> or --remote <path>")
            }

            show_status(&api_client, true, 10).await?;
        },
        Command::Abort => todo!(),
        Command::Pause => todo!(),
        Command::Resume => todo!(),
        Command::Status(status_args) => {
            show_status(
                &api_client,
                status_args.watch,
                status_args.period.unwrap_or(10),
            )
            .await?;
        },
        Command::Search(search_args) => {
            execute_search(search_args.timeout.unwrap_or(2), search_args.alt)
                .await?
                .iter()
                .for_each(|p| {
                    println!(
                        "{}",
                        format!(
                            "Found printer \"{}\", ver {} ({})",
                            p.name.as_ref().unwrap_or(&"<unknown>".to_string()),
                            p.ver.as_ref().unwrap_or(&"<unknown>".to_string()),
                            p.ip
                        )
                    )
                });
        },
    }

    Ok(())
}
