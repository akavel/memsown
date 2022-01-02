use std::collections::HashMap;

use regex::Regex;

use backer::config::{self, *};
use backer::interlude::*;

fn main() {
    iprintln!("SAMPLE:\n" toml::to_string(&Config {
        markers: Markers{
            disk: Vec::new(),
        },
        date_path: HashMap::from([
            ("marker-x".to_string(), vec![
                DatePath {
                    path: Regex::new(r"/(20\d\d)(\d\d)(\d\d)_(\d\d)(\d\d)(\d\d).jpg").unwrap(),
                    date: "$1-$2-$3 $4:$5:$6".to_string(),
                },
            ]),
        ]),
    }).unwrap() "\n");

    match config::read("backer.toml") {
        Err(err) => ieprintln!("Error: " error_chain(&err) "."),
        Ok(config) => iprintln!("CONFIG: " config;?),
    }
}
