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

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_folder: default_sync_folder(),
            device_id: default_device_id(),
        }
    }
}

pub fn default_sync_folder() -> PathBuf {
    project_directories().data_dir().to_path_buf()
}

pub fn default_device_id() -> String {
    hostname::get().unwrap().to_string_lossy().to_string()
}

pub fn project_directories() -> directories::ProjectDirs {
    directories::ProjectDirs::from("xyz", "geemili", "augr").unwrap()
}

pub fn load_config(path: &Path) -> Result<Config> {
    let conf_str = read_to_string(path)
        .with_context(|| format!("Failed to read configuration at {:?}", path))?;

    let conf_doc = conf_str
        .parse::<Document>()
        .context("Invalid configuration file")?;

    let sync_folder = conf_doc["sync_folder"]
        .as_str()
        .map(|p| p.into())
        .unwrap_or_else(default_sync_folder);
    let device_id = conf_doc["device_id"]
        .as_str()
        .map(|d| d.into())
        .unwrap_or_else(default_device_id);

    Ok(Config {
        sync_folder,
        device_id,
    })
}
