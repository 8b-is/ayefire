//! Integer cell coordinates preserve every character within a pane.
use std::fmt::Write;

struct Pane<'a> {
    title: &'a str,
    body: &'a str,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Pane<'_> {
    fn cell(&self, x: usize, y: usize) -> Option<char> {
        if x < self.x || y < self.y || x >= self.x + self.width || y >= self.y + self.height {
            return None;
        }
        let column = x - self.x;
        let row = y - self.y;
        if row == 0 {
            return Some(if column == 0 || column + 1 == self.width {
                '+'
            } else {
                '-'
            });
        }
        if row + 1 == self.height {
            return Some(if column == 0 || column + 1 == self.width {
                '+'
            } else {
                '-'
            });
        }
        if column == 0 || column + 1 == self.width {
            return Some('|');
        }
        if column == 1 {
            return Some(' ');
        }
        let text = if row == 1 {
            self.title
        } else if row == 3 {
            self.body
        } else {
            ""
        };
        // Socket output is data: never allow terminal-control escape sequences.
        Some(
            text.chars()
                .map(|c| {
                    if c.is_ascii_graphic() || c == ' ' {
                        c
                    } else {
                        '?'
                    }
                })
                .nth(column - 2)
                .unwrap_or(' '),
        )
    }
}

pub fn screen(
    columns: usize,
    rows: usize,
    capsules: &str,
    heartbeat: &str,
    socket: &str,
    color: bool,
) -> String {
    let columns = columns.clamp(32, 160);
    let rows = rows.clamp(16, 60);
    let width = columns - 4;
    let panes = [
        Pane {
            title: "Capsules",
            body: capsules,
            x: 2,
            y: 1,
            width,
            height: 6,
        },
        Pane {
            title: "Heartbeat",
            body: heartbeat,
            x: 2,
            y: 7,
            width,
            height: 6,
        },
        Pane {
            title: "Local Unix socket",
            body: socket,
            x: 2,
            y: 13,
            width,
            height: 6,
        },
    ];
    let mut output = String::new();
    if color {
        output.push_str("\x1b[2J\x1b[H");
    }
    // At least 20 rows keep all three panes visible even on a small viewport.
    for y in 0..rows.max(20) {
        for x in 0..columns {
            if color {
                let distance = x.abs_diff(columns / 2) + y.abs_diff(rows / 2);
                let light = 24usize.saturating_sub(distance / 3);
                let _ = write!(
                    output,
                    "\x1b[48;2;{};{};{}m\x1b[38;2;232;240;248m",
                    8 + light,
                    16 + light,
                    24 + light
                );
            }
            output.push(panes.iter().find_map(|pane| pane.cell(x, y)).unwrap_or(' '));
        }
        if color {
            output.push_str("\x1b[0m");
        }
        output.push('\n');
    }
    output.push_str("AyeFire | local socket status | one snapshot per run\n");
    output
}

#[cfg(test)]
mod tests {
    use super::screen;

    #[test]
    fn complete_labels_and_replies_survive_cell_mapping() {
        let text = screen(
            78,
            24,
            r#"{"capsules":["genesis"]}"#,
            "pong",
            "/tmp/ayeosd.sock",
            false,
        );
        for expected in [
            "Capsules",
            "Heartbeat",
            "Local Unix socket",
            "genesis",
            "pong",
            "/tmp/ayeosd.sock",
        ] {
            assert!(text.contains(expected), "missing {expected}");
        }
        assert_eq!(text.lines().next().unwrap().len(), 78);
    }

    #[test]
    fn response_cannot_inject_terminal_escapes() {
        let text = screen(32, 16, "\x1b[2J\nmalicious", "pong", "socket", false);
        assert!(!text.contains('\x1b'));
        assert!(text.contains("Heartbeat"));
        assert!(text.contains("pong"));
    }
}
