use crate::{
    api::{ApiService, FileExt, PlacingType},
    context::{add_to_context, remove_from_context},
    search::execute_search,
    status::show_status,
};
use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use dialoguer::{Select, theme::ColorfulTheme};
use msla_core::{
    logging,
    types::cli::args::{Args, Command},
};
use notify_rust::{Notification, Urgency};
use std::{net::IpAddr, path::PathBuf};
use tracing::{error, info};

mod api;
mod context;
mod search;
mod status;

#[cfg(windows)]
fn set_console_visible(visible: bool) {
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOW, ShowWindow};

    unsafe {
        let hwnd = GetConsoleWindow();

        if !hwnd.is_invalid() {
            let _ = ShowWindow(hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
    }
}

#[cfg(not(windows))]
fn set_console_visible(visible: bool) {}

async fn get_printers(alt_scan: bool, port: u16) -> Result<IpAddr> {
    let found = execute_search(1, alt_scan, port).await?;

    if found.is_empty() {
        return Err(anyhow!(
            "Printers in this local network not found. Specify IP `--host <ip>`"
        ));
    }

    if found.len() == 1 {
        let found = found.first().unwrap();
        info!(
            "Found printer \"{}\", ver {} ({})",
            found.name.as_ref().unwrap_or(&"<unknown>".to_string()),
            found.ver.as_ref().unwrap_or(&"<unknown>".to_string()),
            found.ip
        );
        return Ok(found.ip);
    }

    set_console_visible(true);

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
        .with_prompt(
            format!("{} printers found, select one", found.len())
                .bold()
                .to_string(),
        )
        .items(options)
        .default(0)
        .interact()?;

    Ok(found.get(selection).unwrap().ip)
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_logger(tracing::Level::INFO, false);
    let args = Args::parse();

    if args.from_context_menu {
        set_console_visible(false);
    }

    match execute(&args).await {
        Ok(()) if args.from_context_menu => {
            Notification::new()
                .summary("Open MSLA")
                .body("Command successfully sent!")
                .appname("msla-cli")
                .show()?;
            return Ok(());
        },

        Err(e) if args.from_context_menu => {
            Notification::new()
                .summary("Open MSLA")
                .body(&e.to_string())
                .appname("msla-cli")
                .urgency(Urgency::Critical)
                .show()?;
            return Err(e);
        },

        r => return r,
    }
}

async fn execute(args: &Args) -> Result<()> {
    match &args.command {
        Command::Search(s) => {
            execute_search(s.timeout.unwrap_or(2), s.alt, args.scan_port.unwrap_or(710))
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

            return Ok(());
        },
        Command::ContextRegister => {
            add_to_context("Send to printer").await?;
            return Ok(());
        },
        Command::ContextUnregister => {
            remove_from_context().await?;
            return Ok(());
        },

        command => {
            let api_client = ApiService::new(
                args.host.clone().unwrap_or(
                    get_printers(args.alt_scan, args.scan_port.unwrap_or(710))
                        .await?
                        .to_string(),
                ),
                args.port.unwrap_or(709),
            );

            match command {
                Command::Start(start_args) => {
                    let ext = if start_args.zip {
                        FileExt::Zip
                    } else {
                        error!("Please, specify the file type: --zip or others");
                        return Ok(());
                    };

                    if let Some(local) = &start_args.local {
                        api_client
                            .start_print(PlacingType::Local, ext, PathBuf::from(local))
                            .await?;
                    } else if let Some(remote) = &start_args.remote {
                        api_client
                            .start_print(PlacingType::Remote, ext, PathBuf::from(remote))
                            .await?;
                    } else {
                        error!("Please, specify the file path: --local <path> or --remote <path>")
                    }

                    if start_args.watch {
                        show_status(&api_client, true, 10).await?;
                    }
                },
                Command::Abort => api_client.abort().await?,
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
                Command::Home => api_client.home().await?,
                Command::DisableStepper => api_client.disable_stepper().await?,
                Command::ShowPreview => {
                    info!("Open url: {}/preview", api_client.url)
                },

                Command::Search(_) | Command::ContextRegister | Command::ContextUnregister => {
                    unreachable!()
                },
            }
        },
    }

    Ok(())
}
