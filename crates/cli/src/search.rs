use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::net::UdpSocket;

fn find_local_network() -> Option<(Ipv4Addr, u32)> {
    let interfaces = if_addrs::get_if_addrs().ok()?;
    const EXCLUDE_NAMES: [&str; 4] = ["vpn", "wsl", "loopback", "tun"];

    for iface in interfaces {
        if iface.is_loopback()
            || EXCLUDE_NAMES
                .iter()
                .any(|e| iface.name.to_lowercase().contains(e))
        {
            continue;
        }

        if let if_addrs::IfAddr::V4(v4) = iface.addr {
            let ip = v4.ip;
            let mask = v4.netmask;
            let mask_bits = u32::from(mask).count_ones();

            return Some((ip, mask_bits));
        }
    }
    None
}

fn find_broadcast() -> Option<IpAddr> {
    let interfaces = if_addrs::get_if_addrs().ok()?;
    const EXCLUDE_NAMES: [&str; 4] = ["vpn", "wsl", "loopback", "tun"];
    for iface in interfaces {
        if iface.is_loopback() {
            continue;
        }
        if let if_addrs::IfAddr::V4(v4_addr) = iface.addr {
            if let Some(broadcast) = v4_addr.broadcast
                && !EXCLUDE_NAMES
                    .iter()
                    .any(|e| iface.name.to_lowercase().contains(e))
            {
                return Some(IpAddr::V4(broadcast));
            }
        }
    }
    None
}

pub struct FoundPrinter {
    pub ip: IpAddr,
    pub name: Option<String>,
    pub ver: Option<String>,
}

/// Execute search operation
pub async fn execute_search(interval: u64, alt: bool, port: u16) -> Result<Vec<FoundPrinter>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0").await?;

    socket.set_broadcast(true)?;

    let packet = [
        0xAA, 0x55, 0x01, // DISCOVER
    ];

    if alt {
        // Unicast scan
        let (local_ip, prefix) = find_local_network().expect("Cannot find local network");

        if prefix > 30 {
            anyhow::bail!("Network is too small: /{prefix}");
        }

        let ip = u32::from(local_ip);

        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };

        let network = ip & mask;
        let broadcast = network | !mask;

        let first_host = network + 1;
        let last_host = broadcast - 1;

        for host in first_host..=last_host {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::from(host)), port);

            socket.send_to(&packet, addr).await?;
        }
    } else {
        // Broadcast
        let broadcast_ip = find_broadcast().expect("Cannot find broadcast address");

        socket.set_broadcast(true)?;

        socket
            .send_to(&packet, SocketAddr::new(broadcast_ip, port))
            .await?;
    }
    let mut buf = [0u8; 1500];
    let deadline = Instant::now() + Duration::from_secs(interval);

    let mut found: Vec<FoundPrinter> = Vec::new();
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        let received = tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await;
        if let Ok(Ok((size, addr))) = received
            && buf.starts_with(&[0xAA, 0x55, 0x02])
        {
            let mut reader = 3;
            let name = buf
                .get(reader)
                .filter(|&&len| size >= 4 + len as usize)
                .and_then(|&len| {
                    let len = (len + 1) as usize;
                    let r = buf.get(reader + 1..reader + len);
                    reader += len;
                    r
                })
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned());

            let ver = buf
                .get(reader)
                .filter(|&&len| size >= 4 + len as usize)
                .and_then(|&len| {
                    let len = (len + 1) as usize;
                    let r = buf.get(reader + 1..reader + len);
                    reader += len;
                    r
                })
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned());

            found.push(FoundPrinter {
                ip: addr.ip(),
                name,
                ver,
            });
        }
    }

    Ok(found)
}
