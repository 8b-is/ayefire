//! Compile an explicitly supplied snapshot; no downloads, privilege, or apply step.
use crate::cidr::Network;
use std::collections::BTreeSet;

pub const MAX_BYTES: usize = 524_288;
pub const MAX_RULES: usize = 10_000;

#[derive(Debug)]
pub struct Policy {
    networks: BTreeSet<Network>,
}

impl Policy {
    pub fn parse(snapshot: &str) -> Result<Self, String> {
        if snapshot.len() > MAX_BYTES {
            return Err("snapshot exceeds 512 KiB".into());
        }
        let mut networks = BTreeSet::new();
        let mut count = 0usize;
        for (number, line) in snapshot.lines().enumerate() {
            let value = line.split('#').next().unwrap_or("").trim();
            if value.is_empty() {
                continue;
            }
            count += 1;
            if count > MAX_RULES {
                return Err("snapshot exceeds 10000 entries".into());
            }
            networks.insert(
                Network::parse(value).map_err(|error| format!("line {}: {error}", number + 1))?,
            );
        }
        Ok(Self { networks })
    }

    pub fn len(&self) -> usize {
        self.networks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.networks.is_empty()
    }

    pub fn nftables(&self) -> String {
        let mut output = format!("# AyeFire reviewed snapshot: {} normalized networks\n# Empty sets provide no threat blocking. No subscription or feed authenticity is implied.\n", self.len());
        // One nft -f transaction replaces only the reserved AyeFire table, including sets.
        output
            .push_str("add table inet ayefire\ndelete table inet ayefire\ntable inet ayefire {\n");
        for (name, kind, v4) in [
            ("denied_v4", "ipv4_addr", true),
            ("denied_v6", "ipv6_addr", false),
        ] {
            output.push_str(&format!(
                "  set {name} {{\n    type {kind}\n    flags interval\n    auto-merge\n"
            ));
            let entries: Vec<_> = self
                .networks
                .iter()
                .filter(|network| network.address.is_ipv4() == v4)
                .map(Network::notation)
                .collect();
            if !entries.is_empty() {
                output.push_str(&format!("    elements = {{ {} }}\n", entries.join(", ")));
            }
            output.push_str("  }\n");
        }
        for hook in ["input", "output", "forward"] {
            output.push_str(&format!(
                "  chain {hook} {{\n    type filter hook {hook} priority 10; policy accept;\n"
            ));
            for (family, set) in [("ip", "denied_v4"), ("ip6", "denied_v6")] {
                if hook != "output" {
                    output.push_str(&format!("    {family} saddr @{set} counter drop\n"));
                }
                if hook != "input" {
                    output.push_str(&format!("    {family} daddr @{set} counter drop\n"));
                }
            }
            output.push_str("  }\n");
        }
        output.push_str("}\n");
        output
    }
}

#[cfg(test)]
mod tests {
    use super::{Policy, MAX_BYTES, MAX_RULES};
    #[test]
    fn deduplicates_and_renders_deterministically() {
        let a =
            Policy::parse("# i1 snapshot\n203.0.113.5/24\n203.0.113.1/24\n2001:db8::1\n").unwrap();
        let b = Policy::parse("2001:db8::1\n203.0.113.0/24").unwrap();
        assert_eq!(a.len(), 2);
        assert_eq!(a.nftables(), b.nftables());
        assert!(!a.nftables().contains("flush ruleset"));
    }
    #[test]
    fn invalid_entry_rejects_whole_snapshot() {
        assert!(Policy::parse("203.0.113.1\n192.168.1.1").is_err());
    }
    #[test]
    fn enforces_input_bounds_even_for_duplicate_lines() {
        assert!(Policy::parse(&"#".repeat(MAX_BYTES + 1)).is_err());
        assert!(Policy::parse(&"203.0.113.1\n".repeat(MAX_RULES + 1)).is_err());
    }
    #[test]
    fn empty_feed_never_claims_protection() {
        let policy = Policy::parse("# no entries\n").unwrap();
        assert!(policy.is_empty());
        assert!(policy.nftables().contains("0 normalized networks"));
    }
}
