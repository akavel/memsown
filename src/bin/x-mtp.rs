use anyhow::Result;
use winmtp::{Provider as MtpProvider, WindowsError};

use backer::interlude::*;

fn main() {
    if let Err(err) = run() {
        ieprintln!("error: " error_chain(&err));
    }
}

fn run() -> Result<()> {
    let provider = MtpProvider::new().unwrap();
    let devices = provider.enumerate_devices().unwrap();
    iprintln!("Enumerating devices:");
    for dev in devices {
        iprintln!("id:" dev.device_id() ", name: " dev.friendly_name());
    }
    iprintln!("(enumerating done.)");
    Ok(())
}
