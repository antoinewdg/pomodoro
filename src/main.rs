mod client;
mod common;
mod daemon;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let is_daemon = args.get(1).map(|v| v == "daemon").unwrap_or(false);
    if is_daemon {
        daemon::main()?;
    } else {
        match client::main(&args[1..]) {
            Ok(_) => (),
            Err(error) => {
                eprintln!("{}", error);
                std::process::exit(1);
            }
        }
    };
    Ok(())
}
