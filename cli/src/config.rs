use anyhow::{Context, Result};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use toml_edit::Document;

pub struct Config {
    pub sync_folder: PathBuf,
    pub device_id: String,
}

pub fn load_config(path: &Path) -> Result<Config> {
    let conf_str = read_to_string(path)
        .with_context(|| format!("Failed to read configuration at {}", path.display()))?;

    let conf_doc = conf_str
        .parse::<Document>()
        .context("Invalid configuration file")?;

    let sync_folder = conf_doc["sync_folder"].as_str().unwrap().into();
    let device_id = conf_doc["device_id"].as_str().unwrap().into();

    Ok(Config {
        sync_folder,
        device_id,
    })
}
