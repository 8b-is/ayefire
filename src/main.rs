//! A read-only spatial terminal for the local ayeOS Unix socket.
mod render;
mod transport;

use std::io::{self, Write};
use std::time::Duration;

fn main() {
    let socket = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("AYEOSD_SOCK").ok())
        .unwrap_or_else(|| "/tmp/ayeosd.sock".into());
    let query = |command| {
        transport::request(&socket, command, Duration::from_secs(2))
            .unwrap_or_else(|error| format!("Unavailable: {error}"))
    };
    let capsules = query("list");
    let heartbeat = query("ping");
    let columns = dimension("COLUMNS", 78, 32, 160);
    let rows = dimension("LINES", 24, 16, 60);
    let screen = render::screen(columns, rows, &capsules, &heartbeat, &socket, true);
    if let Err(error) = io::stdout().write_all(screen.as_bytes()) {
        if error.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("AyeFire output: {error}");
            std::process::exit(1);
        }
    }
}

fn dimension(name: &str, fallback: usize, minimum: usize, maximum: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback)
        .clamp(minimum, maximum)
}
