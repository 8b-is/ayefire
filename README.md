# AyeFire

A dependency-free Rust terminal showing ayeOS capsule storage, heartbeat, and
its local Unix socket. Each invocation draws one status snapshot.

```sh
cargo run -- /path/to/ayeosd.sock
# Or set AYEOSD_SOCK; the default is /tmp/ayeosd.sock.
```

Start a compatible `ayeosd` separately. `list` returns capsule metadata and
`ping` returns `pong`. This terminal does not generate conversational replies.
The default ayeOS genesis capsule is a matrix demonstration, not a language model.

The renderer uses exact integer cell coordinates and integer background shading.
It preserves adjacent characters and labels, sanitizes terminal-control bytes,
and bounds replies to 64 KiB with socket read/write timeouts. `COLUMNS` and
`LINES` can set the viewport; rendering requires at least 32 columns and 20 rows.
The client has no TCP/UDP transport or external dependencies. That fact alone
is not an operating-system firewall or a guarantee about the daemon or other apps.

For a user service, `deploy/ayefire-ayeosd.service` restricts the daemon to
`AF_UNIX`, disables its optional `AYEOSD_TCP` setting, and creates a private
runtime directory. Install the matching ayeOS daemon binary at
`~/.local/libexec/ayefire/ayeosd` before enabling the service. This unit requires
an ayeOS daemon that remains running after stdin closes (tested with ayeOS
`e7910256c7ac19bf6ff4f4dfd228e81e1e3b99bf`).

```sh
mkdir -p ~/.config/systemd/user
cp deploy/ayefire-ayeosd.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now ayefire-ayeosd.service
cargo run -- "$XDG_RUNTIME_DIR/ayefire/ayeosd.sock"
```

Verify locally:

```sh
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo tree
```

The regression tests cover complete pane text, terminal-control sanitization,
and a real Unix-socket request/reply exchange. The terminal uses bounded socket
reads; wrap execution with a process timeout when consuming an untrusted daemon.

Original AyeFire implementation: peterlodri-sec/ayefire, commit `2d37d22`.
