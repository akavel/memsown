use std::path::{Path, PathBuf};
use std::fmt::Display;
use std::fs;

use anyhow::{Context, Result};
use ifmt::iformat as ifmt;
use serde::Deserialize;


#[derive(Deserialize)]
pub struct Config {
    pub markers: Markers,
}

#[derive(Deserialize)]
pub struct Markers {
    pub disk: Vec<PathBuf>,
}

pub fn read<P: AsRef<Path> + Display>(path: P) -> Result<Config> {
    let raw = fs::read_to_string(&path).context("reading config file")?;
    let config = toml::from_str(&raw).with_context(|| ifmt!("reading config file '{path}'"))?;
    Ok(config)
}
