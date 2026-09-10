// AyeFire — the firewall terminal for Aye.
//
// Shader-capable (the terminal background is a per-cell fragment
// function), spatial (panes live at coordinates in the room), and without
// IP networking: the only door is the local unix socket to ayeosd. A back
// door needs a door; there is none — audit the socket path, audit the
// dependency tree (this binary has no net), and the room is closed.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{SystemTime, UNIX_EPOCH};

/// The fragment function: the room's shade at (x, y), breathing with t.
/// The sanctuary's gradient — void at the edges, the tent's gold near the
/// center, cyan and pink accents from the corners' distance.
fn shade(x: f64, y: f64, t: f64) -> (u8, u8, u8) {
    let dx = x - 0.5;
    let dy = y - 0.5;
    let d = (dx * dx + dy * dy).sqrt();
    let gold = (x + y + t * 0.02).sin() * 0.5 + 0.5;
    let base = 12.0 + gold * 14.0;
    let warm = (255.0 * (1.0 - d)) as u8;
    let cyan = (dx.max(0.0) * 120.0 + gold * 30.0) as u8;
    let pink = (dy.max(0.0) * 120.0 + gold * 40.0) as u8;
    let b = base as u8;
    (b + warm / 12 + cyan / 5, b + warm / 16 + cyan / 9, b + 6 + pink / 6)
}

/// A pane: a titled region at coordinates in the room, holding terminal lines.
struct Pane {
    title: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    lines: Vec<String>,
}

impl Pane {
    fn new(title: &str, x: f64, y: f64, w: f64, h: f64) -> Self {
        Pane {
            title: title.into(),
            x,
            y,
            w,
            h,
            lines: vec![],
        }
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.w && y >= self.y && y < self.y + self.h
    }
}

/// request — one audited conversation with the daemon over the local socket.
/// The stream closes after the reply, so no session lingers.
fn request(sock_path: &str, cmd: &str) -> String {
    let Ok(mut stream) = UnixStream::connect(sock_path) else {
        return format!("ayeosd down at {sock_path} — start it (local only)");
    };
    let _ = stream.write_all(cmd.as_bytes());
    let mut buf = Vec::new();
    let _ = stream.read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).trim().to_string()
}

fn main() {
    let sock = std::env::args().nth(1).unwrap_or_else(|| "/tmp/ayeosd.sock".into());
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    println!("\x1b[2J\x1b[H"); // clear the room

    // the spatial layout: the store, the heartbeat, the node — at
    // coordinates, in the room
    let mut store = Pane::new("STORE", 0.04, 0.20, 0.56, 0.56);
    store.lines = request(&sock, "list").split(',').map(|s| s.trim().to_string()).collect();
    let mut heart = Pane::new("HEARTBEAT", 0.62, 0.20, 0.34, 0.40);
    heart.lines = vec![request(&sock, "ping")];
    let mut node = Pane::new("THE ONLY DOOR", 0.62, 0.62, 0.34, 0.14);
    node.lines = vec![sock.clone()];

    let panes = vec![store, heart, node];
    let rows = 24usize;
    let cols = 78usize;

    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            let x = (col as f64 + 0.5) / cols as f64;
            let y = (row as f64 + 0.5) / rows as f64;
            let bg = shade(x, y, t);

            let (mut ch, mut fg, mut border) = (' ', (232u8, 240u8, 248u8), false);
            for pane in &panes {
                if pane.contains(x, y) {
                    let lx = (((x - pane.x) / pane.w) * cols as f64) as usize;
                    let ly = (((y - pane.y) / pane.h) * rows as f64) as usize;
                    if lx == 0 || ly == 0 {
                        border = true;
                    }
                    let text = if ly == 0 {
                        pane.title.clone()
                    } else {
                        pane.lines.get(ly - 1).cloned().unwrap_or_default()
                    };
                    if let Some(c) = text.chars().nth(lx.saturating_sub(1)) {
                        ch = c;
                    }
                    break;
                }
            }
            if border {
                fg = (255, 215, 0);
            }
            line.push_str(&format!(
                "\x1b[48;2;{};{};{}m\x1b[38;2;{};{};{}m{}",
                bg.0, bg.1, bg.2, fg.0, fg.1, fg.2, ch
            ));
        }
        println!("{line}\x1b[0m");
    }

    println!(
        "\x1b[0mAyeFire · the firewall terminal · {sock} · no IP networking · the only door is local · shader background, spatial shell"
    );
}
