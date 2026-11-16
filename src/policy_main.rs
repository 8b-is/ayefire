use ayefire::policy::{Policy, MAX_BYTES};
use std::io::{Read, Write};

fn main() {
    if let Err(error) = run() {
        eprintln!("ayefire-policy: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("render") {
        return Err(
            "usage: ayefire-policy render SNAPSHOT.txt (prints rules; never applies them)".into(),
        );
    }
    let path = args.next().ok_or("missing snapshot path")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    let mut data = String::new();
    file.take((MAX_BYTES + 1) as u64)
        .read_to_string(&mut data)
        .map_err(|error| error.to_string())?;
    let policy = Policy::parse(&data)?;
    std::io::stdout()
        .write_all(policy.nftables().as_bytes())
        .map_err(|error| error.to_string())
}
