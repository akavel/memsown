use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::interlude::*;


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub markers: Markers,
    pub date_path: HashMap<String, Vec<DatePath>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Markers {
    pub disk: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatePath {
    pub date: String,
    #[serde(with = "serde_regex")]
    pub path: Regex,
}

pub fn read<P: AsRef<Path> + Display>(path: P) -> Result<Config> {
    let raw = fs::read_to_string(&path).context("reading config file")?;
    let config = toml::from_str(&raw).with_context(|| ifmt!("reading config file '{path}'"))?;
    Ok(config)
}
