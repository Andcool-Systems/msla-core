use colored::Colorize;
use std::time::{Duration, Instant};

use anyhow::Result;
use msla_core::config;
use tokio::net::UdpSocket;

/// Execute search operation
pub async fn execute_search(interval: u64) -> Result<()> {
    let dur = Duration::from_secs(interval);
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;

    let packet = [
        0xAA, 0x55, 0x01, // DISCOVER
    ];

    let config = config::get_config().await;
    socket
        .send_to(
            &packet,
            format!("255.255.255.255:{}", config.broadcast_listener.port),
        )
        .await?;
    let mut buf = [0u8; 1500];

    let deadline = Instant::now() + dur;
    loop {
        let remaining = deadline - Instant::now();
        let received = tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await;

        let Ok(Ok((_, addr))) = received else {
            break;
        };

        println!("Printer found at {}", addr.ip().to_string().bold());
    }

    Ok(())
}
