use std::net::Ipv6Addr;

pub enum PeerAddress {
    Id(String),
    Ipv6(Ipv6Addr),
}

pub fn parse_peer(input: &str) -> PeerAddress {
    if input.contains(':') {
        if let Ok(addr) = input.parse::<Ipv6Addr>() {
            return PeerAddress::Ipv6(addr);
        }
    }
    PeerAddress::Id(input.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv6() {
        match parse_peer("::1") {
            PeerAddress::Ipv6(addr) => assert_eq!(addr.to_string(), "::1"),
            _ => panic!("Expected IPv6"),
        }

        match parse_peer("2001:db8::1") {
            PeerAddress::Ipv6(addr) => assert_eq!(addr.to_string(), "2001:db8::1"),
            _ => panic!("Expected IPv6"),
        }
    }

    #[test]
    fn test_parse_id() {
        match parse_peer("alice") {
            PeerAddress::Id(id) => assert_eq!(id, "alice"),
            _ => panic!("Expected ID"),
        }

        match parse_peer("user123") {
            PeerAddress::Id(id) => assert_eq!(id, "user123"),
            _ => panic!("Expected ID"),
        }
    }
}
