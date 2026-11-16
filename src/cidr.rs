//! Parse and normalize numeric networks without shell syntax or DNS lookups.
use std::net::IpAddr;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Network {
    pub address: IpAddr,
    pub prefix: u8,
}

impl Network {
    pub fn parse(text: &str) -> Result<Self, String> {
        let (host, suffix) = text
            .split_once('/')
            .map_or((text, None), |(a, b)| (a, Some(b)));
        let address: IpAddr = host.parse().map_err(|_| "expected numeric IP address")?;
        let width = if address.is_ipv4() { 32 } else { 128 };
        let prefix: u8 = match suffix {
            Some(p) => p.parse().map_err(|_| "invalid prefix length")?,
            None => width,
        };
        if prefix > width {
            return Err("prefix exceeds address width".into());
        }
        let number = match address {
            IpAddr::V4(a) => u32::from(a) as u128,
            IpAddr::V6(a) => u128::from(a),
        };
        let mask = if prefix == 0 {
            0
        } else {
            u128::MAX << (width - prefix)
        };
        let network = number & mask;
        let host_mask = if prefix == 0 && width == 128 {
            u128::MAX
        } else {
            (1u128 << (width - prefix)) - 1
        };
        let end = network | host_mask;
        let excluded: &[(&str, u8)] = if width == 32 {
            &[
                ("0.0.0.0", 8),
                ("10.0.0.0", 8),
                ("100.64.0.0", 10),
                ("127.0.0.0", 8),
                ("169.254.0.0", 16),
                ("172.16.0.0", 12),
                ("192.168.0.0", 16),
                ("224.0.0.0", 3),
            ]
        } else {
            &[
                ("::", 96),
                ("::ffff:0:0", 96),
                ("fc00::", 7),
                ("fe80::", 10),
                ("ff00::", 8),
            ]
        };
        for (base, bits) in excluded {
            let base: IpAddr = base.parse().expect("literal reserved range");
            let start = match base {
                IpAddr::V4(a) => u32::from(a) as u128,
                IpAddr::V6(a) => u128::from(a),
            };
            let last = start | ((1u128 << (width - bits)) - 1);
            if network <= last && end >= start {
                return Err("network overlaps a local, private, shared or multicast range".into());
            }
        }
        Ok(Self {
            address: if width == 32 {
                IpAddr::V4((network as u32).into())
            } else {
                IpAddr::V6(network.into())
            },
            prefix,
        })
    }

    pub fn notation(&self) -> String {
        format!("{}/{}", self.address, self.prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::Network;
    #[test]
    fn normalizes_host_bits_for_both_families() {
        assert_eq!(
            Network::parse("203.0.113.99/24").unwrap().notation(),
            "203.0.113.0/24"
        );
        assert_eq!(
            Network::parse("2001:db8::99/64").unwrap().notation(),
            "2001:db8::/64"
        );
    }
    #[test]
    fn rejects_injection_and_malformed_networks() {
        for input in [
            "1.2.3.4; flush ruleset",
            "example.org",
            "1.2.3.4/33",
            "2001:db8::/129",
            "1.2.3.4/2/3",
            "fe80::1%eth0",
        ] {
            assert!(Network::parse(input).is_err(), "{input}");
        }
    }
    #[test]
    fn rejects_local_ranges_and_supernets_that_cover_them() {
        for input in [
            "0.0.0.0/0",
            "::/0",
            "127.0.0.1",
            "10.0.0.1",
            "100.64.1.2",
            "169.254.1.1",
            "172.16.1.1",
            "192.168.1.1",
            "224.0.0.1",
            "8.0.0.0/5",
            "::1",
            "::ffff:127.0.0.1",
            "fc00::/7",
            "fe80::1",
            "ff02::1",
        ] {
            assert!(Network::parse(input).is_err(), "{input}");
        }
    }
}
