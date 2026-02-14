use anyhow::{bail, Context, Result};
use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct Ipv6Info {
    pub addr: Ipv6Addr,
    pub interface: String,
    pub is_temporary: bool,
    pub scope: Ipv6Scope,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ipv6Scope {
    LinkLocal,
    UniqueLocal,
    Global,
}

impl Ipv6Info {
    fn score(&self) -> u32 {
        let mut score = 0;

        // Prefer non-temporary
        if !self.is_temporary {
            score += 1000;
        }

        // Prefer by scope: Global > ULA > Link-local
        score += match self.scope {
            Ipv6Scope::Global => 100,
            Ipv6Scope::UniqueLocal => 50,
            Ipv6Scope::LinkLocal => 10,
        };

        score
    }
}

fn classify_ipv6(addr: &Ipv6Addr) -> Ipv6Scope {
    if addr.is_loopback() {
        return Ipv6Scope::LinkLocal;
    }

    let segments = addr.segments();

    // Link-local: fe80::/10
    if segments[0] & 0xffc0 == 0xfe80 {
        return Ipv6Scope::LinkLocal;
    }

    // ULA: fc00::/7
    if segments[0] & 0xfe00 == 0xfc00 {
        return Ipv6Scope::UniqueLocal;
    }

    // Global unicast: 2000::/3
    if segments[0] & 0xe000 == 0x2000 {
        return Ipv6Scope::Global;
    }

    Ipv6Scope::LinkLocal
}

fn is_temporary_address(addr: &Ipv6Addr) -> bool {
    // Read /proc/net/if_inet6 to check flags
    if let Ok(content) = std::fs::read_to_string("/proc/net/if_inet6") {
        let addr_hex = format!("{:032x}", u128::from(*addr));

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[0] == addr_hex {
                // flags are in parts[4] as hex
                if let Ok(flags) = u32::from_str_radix(parts[4], 16) {
                    // 0x01 = temporary address flag
                    return flags & 0x01 != 0;
                }
            }
        }
    }
    false
}

pub fn get_all_ipv6() -> Result<Vec<Ipv6Info>> {
    let interfaces = if_addrs::get_if_addrs()?;
    let mut addrs = Vec::new();

    for iface in interfaces {
        if let IpAddr::V6(addr) = iface.addr.ip() {
            if addr.is_loopback() || addr.is_multicast() {
                continue;
            }

            let scope = classify_ipv6(&addr);
            let is_temporary = is_temporary_address(&addr);

            addrs.push(Ipv6Info {
                addr,
                interface: iface.name.clone(),
                is_temporary,
                scope,
            });
        }
    }

    if addrs.is_empty() {
        bail!("No suitable IPv6 address found")
    }

    // Sort by score (best first)
    addrs.sort_by_key(|a| std::cmp::Reverse(a.score()));

    Ok(addrs)
}

pub fn get_local_ipv6() -> Result<Ipv6Addr> {
    let addrs = get_all_ipv6()?;
    Ok(addrs[0].addr)
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
