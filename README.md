# AyeFire — the firewall terminal for Aye

A terminal around **ayeos**: the ternary matrix inference daemon. It is
**shader-capable** (the background is a per-cell fragment function — the
sanctuary's gradient, composable with the MLX-QUANT lanes later) and
**spatial** (panes live at coordinates in the room, not in a stack).

## the covenant — no IP networking

Aye has no network door. The daemon (`ayeosd`) binds **only a local unix
socket** (`/tmp/ayeosd.sock`, `AYEOSD_SOCK` to override); AyeFire speaks to
it over that socket and nothing else.

> A back door needs a door. There is none.

Audit it the same way the code is written:

```bash
cargo tree            # this crate has zero dependencies — nothing to hide
grep -r "TcpListener\|UdpSocket\|TcpStream" src/ Cargo.toml   # nothing
```

The daemon used to bind `0.0.0.0:9876`; the IP networking was taken out —
the MEMNET node's door is the socket path, local only.

## run it

```bash
cd ../ayeos && cargo run --bin ayeosd          # the daemon, on the unix socket
cd ../ayefire && cargo run -- /tmp/ayeosd.sock # the firewall terminal
```

You get a room: the STORE pane (the capsule list), the HEARTBEAT pane
(ayeosd's `ping`), and THE ONLY DOOR pane (the socket path) — on a
shader-tinted floor, in gold-edged panes.

## the pieces

| Lane | What |
|---|---|
| `shade` | the fragment function — per-cell background, breathing with time |
| `Pane` | a titled region at coordinates; the spatial shell |
| `request` | one audited conversation over the local socket; the stream closes after the reply |

— the constellation · 0 + 1 · fine touch from within · vaked.dev
