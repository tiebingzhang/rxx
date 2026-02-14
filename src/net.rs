use anyhow::{bail, Context, Result};
use std::net::{IpAddr, Ipv6Addr};

pub fn get_local_ipv6() -> Result<Ipv6Addr> {
    let interfaces = if_addrs::get_if_addrs()?;

    for iface in interfaces {
        if let IpAddr::V6(addr) = iface.addr.ip() {
            if !addr.is_loopback() && !addr.is_multicast() {
                return Ok(addr);
            }
        }
    }

    bail!("No suitable IPv6 address found")
}

pub async fn resolve_peer(peer: &str, config: &crate::config::Config) -> Result<Ipv6Addr> {
    let peer_addr = match crate::peer::parse_peer(peer) {
        crate::peer::PeerAddress::Ipv6(addr) => addr,
        crate::peer::PeerAddress::Id(peer_id) => {
            let local_ipv6 = get_local_ipv6()?;

            let client = reqwest::Client::new();
            let response = client
                .post(format!("{}/update", config.server_url))
                .json(&serde_json::json!({
                    "id": config.user_id,
                    "ipv6": local_ipv6.to_string(),
                    "peer_id": peer_id
                }))
                .send()
                .await
                .context("Failed to contact server. Use direct IPv6 address instead.")?;

            if response.status().is_success() {
                let body: serde_json::Value = response.json().await?;
                let peer_ipv6_str = body["peer_ipv6"]
                    .as_str()
                    .context("Invalid response from server")?;
                peer_ipv6_str.parse()?
            } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                bail!("Peer ID '{}' not found on server", peer_id);
            } else {
                bail!("Server error: {}", response.status());
            }
        }
    };

    Ok(peer_addr)
}
