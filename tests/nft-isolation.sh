#!/usr/bin/env bash
# Run only inside a fresh network namespace; never change the host's firewall.
set -euo pipefail
binary=$(realpath "${1:?usage: nft-isolation.sh PATH_TO_AYEFIRE_POLICY}")
if [[ $EUID -ne 0 ]]; then
  exec sudo -n "$0" "$binary"
fi
exec unshare --net /bin/bash -eu -s -- "$binary" <<'ISOLATED'
binary=$1
scratch=$(mktemp -d /tmp/ayefire-proof.XXXXXXXX)
trap 'rm -rf -- "$scratch"' EXIT
ip link set lo up
ip address add 203.0.113.10/32 dev lo
ip address add 203.0.113.11/32 dev lo
ip -6 address add 2001:db8::10/128 dev lo nodad
ip -6 address add 2001:db8::11/128 dev lo nodad
ping -n -c 1 -W 1 203.0.113.10 >/dev/null
ping -6 -n -c 1 -W 1 2001:db8::10 >/dev/null
nft add table inet unrelated_canary
printf '%s\n' '203.0.113.10' '2001:db8::10' > "$scratch/feed"
"$binary" render "$scratch/feed" > "$scratch/policy.nft"
nft --check --file "$scratch/policy.nft"
nft --file "$scratch/policy.nft"
if ping -n -c 1 -W 1 203.0.113.10 >/dev/null 2>&1; then exit 1; fi
if ping -6 -n -c 1 -W 1 2001:db8::10 >/dev/null 2>&1; then exit 1; fi
ping -n -c 1 -W 1 203.0.113.11 >/dev/null
ping -6 -n -c 1 -W 1 2001:db8::11 >/dev/null
nft list table inet unrelated_canary >/dev/null
nft --file "$scratch/policy.nft"
# Replacement withdraws old entries rather than silently accumulating them.
printf '%s\n' '203.0.113.12' > "$scratch/feed"
"$binary" render "$scratch/feed" > "$scratch/policy.nft"
nft --file "$scratch/policy.nft"
ping -n -c 1 -W 1 203.0.113.10 >/dev/null
ping -6 -n -c 1 -W 1 2001:db8::10 >/dev/null
nft delete table inet ayefire
nft list table inet unrelated_canary >/dev/null
printf '%s\n' 'PASS: IPv4 and IPv6 deny/allow, repeat apply, withdrawal, unrelated table retention, rollback.'
ISOLATED
