use anyhow::Result;
use msla_core::config;
use tokio::net::UdpSocket;
use tracing::debug;

/// Start broadcast listener
pub async fn start_broadcast() -> Result<()> {
    let config = config::get_config().await;
    let socket = UdpSocket::bind((
        config.broadcast_listener.addr.clone(),
        config.broadcast_listener.port,
    ))
    .await?;
    debug!("Listening udp broadcast");

    let mut buf = [0u8; 1500];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let packet = &buf[..len];

        println!("rec: {:?}", packet);

        if packet.len() < 3 {
            continue;
        }

        if packet[0] != 0xAA || packet[1] != 0x55 {
            continue;
        }

        match packet[2] {
            // DISCOVER
            0x01 => {
                debug!("Discovery from {addr}");
                let response = [0xAA, 0x55, 0x02];
                socket.send_to(&response, addr).await?;
            },

            _ => {
                debug!("Unknown command");
            },
        }
    }
}
