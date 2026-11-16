# Supplemental Linux host-policy experiment

The product is **AyeFire, the firewall/router edition of AyeOS**, whose native
packet core lives in [8b-is/aye](https://github.com/8b-is/aye). The proposed i1
managed service is $5 per household per month (`USD`, `500` integer cents).

This repository's standalone terminal remains an auxiliary tool. The
`ayefire-policy` binary is an experimental Linux adapter, not the AyeOS packet
core, subscription implementation, authenticated updater, or router image.
It is retained as a bounded way to test reviewed CIDR policies against the
Linux kernel while the AyeOS edition develops around its existing core.

```sh
cargo build --locked --offline --release --bin ayefire-policy
./target/release/ayefire-policy render reviewed-snapshot.txt > reviewed.nft
./tests/nft-isolation.sh ./target/release/ayefire-policy
```

The binary reads one local file and prints a complete nftables transaction.
It never downloads policies, executes nft, requests root, or changes networking.
The input format is numeric IPv4/IPv6 addresses or CIDRs, one per line, with `#`
comments. Malformed or oversized snapshots are rejected as a whole. Addresses
are normalized and deduplicated; local/private/shared/multicast ranges and
supernets overlapping them are rejected. Input is capped at 512 KiB and 10,000
non-comment entries.

A transaction replaces only the reserved `inet ayefire` table, including its
sets. Other firewall tables remain intact; existing firewall drops still apply.
Do not use that table name for unrelated rules. Use
[nft's atomic file operation](https://wiki.nftables.org/wiki-nftables/index.php/Atomic_rule_replacement)
only after checking ownership, management reachability and rollback on the
intended host. The included test applies rules exclusively in a new network
namespace and proves IPv4/IPv6 deny and allow, repeated application, withdrawal,
retention of an unrelated table, and deletion of the experimental table.

The current `https://i1.is/blocklist` endpoint is a community scanner-IP feed.
Its JSON metadata at `https://i1.is/blocklist.json` reports version, update time,
description and entries. It was empty during this work. That is not evidence of
malware protection or a working managed subscription. The compiler does not
verify signatures, freshness or reputation: its input is an explicitly reviewed
snapshot. Never schedule blind download-and-apply using this prototype.

Before managed activation, the AyeOS edition needs device enrollment under
id1.is, authenticated household administration, signed/versioned policies,
existing-flow revocation, verified IPv6 forwarding, rollback and hardware tests.
The product's local routing and local rules must survive service outages and
cancellation. Paid billing and support remain future work.
