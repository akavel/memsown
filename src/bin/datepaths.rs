use std::convert::TryInto;

use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime};

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
        let tree: Tree = match marker_path.as_path().try_into() {
            Ok(t) => t,
            Err(e) => {
                ieprintln!("Skipping: " e);
                continue;
            }
        };

        let date_paths = config.date_path.remove(&tree.marker);
        'files: for path in tree.iter()? {
            let path = match path {
                Ok(p) => p,
                Err(e) => {
                    ieprintln!("Failed to access file, skipping: " e);
                    continue;
                }
            };
            let relative = relative_slash_path(&tree.root, &path)?;
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
