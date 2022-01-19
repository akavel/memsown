use anyhow::{bail, Result};
use chrono::{NaiveDate, NaiveDateTime};
use path_slash::PathExt;

use backer::config;
use backer::interlude::*;
use backer::scanning::*;

fn main() {
    if let Err(err) = run() {
        ieprintln!("error: " error_chain(&err));
    }
}

const YMD_HMS: &str = "%Y-%m-%d %H:%M:%S";
const YMD: &str = "%Y-%m-%d";

fn run() -> Result<()> {
    let mut config = config::read("backer.toml")?;
    for marker_path in config.markers.disk {
        iprintln!("MARKER: " marker_path;?);
        let tree = match Tree::open(marker_path, &config.date_path) {
            Ok(t) => t,
            Err(e) => {
                ieprintln!("Skipping: " e);
                continue;
            }
        };

        let date_paths = config.date_path.remove(&tree.marker);
        'files: for entry in tree.iter() {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    ieprintln!("Failed to access file, skipping: " e);
                    continue;
                }
            };
            let relative = if let Some(p) = entry.relative_path().to_slash() {
                p
            } else {
                bail!(
                    "Failed to convert path to slash: {:?}",
                    entry.relative_path()
                );
            };
            for date_path in date_paths.iter().flatten() {
                if let Some(found) = date_path.path.captures(&relative) {
                    let mut buf = String::new();
                    found.expand(&date_path.date, &mut buf);
                    let date = NaiveDateTime::parse_from_str(&buf, YMD_HMS).or_else(|_| {
                        NaiveDate::parse_from_str(&buf, YMD).map(|d| d.and_hms(0, 0, 0))
                    });
                    iprintln!("+ " date;? " " relative;?);
                    continue 'files;
                }
            }
            iprintln!(".    " relative;?);
        }
    }
    Ok(())
}
