//! Bounded requests over Unix sockets; no IP transport or dependencies.
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::time::Duration;

const MAX_REPLY: u64 = 65_536;

pub fn request(socket: &str, command: &str, timeout: Duration) -> io::Result<String> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(command.as_bytes())?;
    stream.shutdown(Shutdown::Write)?;
    let mut bytes = Vec::new();
    stream.take(MAX_REPLY + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_REPLY as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "reply exceeds 64 KiB",
        ));
    }
    String::from_utf8(bytes)
        .map(|s| s.trim().to_owned())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use super::request;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;
    use std::time::Duration;

    #[test]
    fn request_ends_write_side_and_reads_full_reply() {
        let path = std::env::temp_dir().join(format!("ayefire-test-{}.sock", std::process::id()));
        let listener = UnixListener::bind(&path).unwrap();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut command = String::new();
            stream.read_to_string(&mut command).unwrap();
            assert_eq!(command, "ping");
            stream.write_all(b"pong\n").unwrap();
        });
        let reply = request(path.to_str().unwrap(), "ping", Duration::from_secs(1));
        worker.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(reply.unwrap(), "pong");
    }
}
