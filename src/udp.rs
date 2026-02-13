use anyhow::{Context, Result};
use std::net::{Ipv6Addr, SocketAddr};
use tokio::net::UdpSocket;
use tokio::time::{interval, timeout, Duration};

const CLIENT_PORT: u16 = 3457;
const SERVER_PORT: u16 = 3458;
const PROBE_PACKET: &[u8] = b"RXX_PROBE";
const PROBE_ACK: &[u8] = b"RXX_PROBE_ACK";
const TIMEOUT_SECS: u64 = 10;
const MAX_RETRIES: u32 = 3;

pub async fn punch_hole(peer_addr: Ipv6Addr, is_server: bool) -> Result<SocketAddr> {
    for attempt in 1..=MAX_RETRIES {
        println!(
            "UDP hole punching attempt {}/{} to {}...",
            attempt, MAX_RETRIES, peer_addr
        );

        match timeout(
            Duration::from_secs(TIMEOUT_SECS),
            try_punch_hole(peer_addr, is_server),
        )
        .await
        {
            Ok(Ok(peer_socket)) => {
                println!("Bidirectional UDP channel established!");
                return Ok(peer_socket);
            }
            Ok(Err(e)) => {
                println!("Attempt {} failed: {}", attempt, e);
            }
            Err(_) => {
                println!(
                    "Attempt {} timed out after {} seconds",
                    attempt, TIMEOUT_SECS
                );
            }
        }

        if attempt < MAX_RETRIES {
            println!("Retrying...");
        }
    }

    anyhow::bail!(
        "Failed to establish UDP channel after {} attempts",
        MAX_RETRIES
    )
}

async fn try_punch_hole(peer_addr: Ipv6Addr, is_server: bool) -> Result<SocketAddr> {
    // Server binds to 3458, client binds to 3457
    let local_port = if is_server { SERVER_PORT } else { CLIENT_PORT };
    let peer_port = if is_server { CLIENT_PORT } else { SERVER_PORT };

    // Bind to IPv6 port
    let socket = UdpSocket::bind(format!("[::]:{}", local_port))
        .await
        .context("Failed to bind UDP socket")?;

    println!("UDP socket bound to [::]{}", local_port);

    let peer_socket = SocketAddr::from((peer_addr, peer_port));

    // Start sending probe packets
    let mut probe_interval = interval(Duration::from_secs(1));
    let mut buf = [0u8; 1024];
    let mut sent_probe = false;
    let mut received_probe = false;

    loop {
        tokio::select! {
            _ = probe_interval.tick() => {
                // Send probe packet
                socket.send_to(PROBE_PACKET, peer_socket)
                    .await
                    .context("Failed to send probe packet")?;
                println!("Sent probe packet to {}", peer_socket);
                sent_probe = true;
            }

            result = socket.recv_from(&mut buf) => {
                let (len, from) = result.context("Failed to receive packet")?;
                let data = &buf[..len];

                if data == PROBE_PACKET {
                    println!("Received probe packet from {}", from);
                    received_probe = true;

                    // Send ACK back
                    socket.send_to(PROBE_ACK, from)
                        .await
                        .context("Failed to send probe ACK")?;
                    println!("Sent probe ACK to {}", from);

                    // If we've received a probe and sent ACK, bidirectional is established
                    // (we can receive from peer, and peer will receive our ACK)
                    return Ok(peer_socket);
                } else if data == PROBE_ACK {
                    println!("Received probe ACK from {}", from);
                    received_probe = true;

                    // Check if bidirectional
                    if sent_probe && received_probe {
                        return Ok(peer_socket);
                    }
                }
            }
        }
    }
}
