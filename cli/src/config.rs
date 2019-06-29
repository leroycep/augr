use serde::Deserialize;
use snafu::ResultExt;
use snafu::Snafu;
use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct Conf {
    pub sync_folder: String,
    pub device_id: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to read configuration from {}: {}", path.display(), source))]
    ReadConfiguration { source: io::Error, path: PathBuf },

    #[snafu(display("Invalid configuration file at {}: {}", path.display(), source))]
    InvalidConfiguration {
        source: toml::de::Error,
        path: PathBuf,
    },
}

pub fn load_config(path: &Path) -> Result<Conf, Error> {
    use std::io::Read;

    let mut conf_str = read_to_string(path).context(ReadConfiguration { path })?;

    let conf = toml::de::from_str(&conf_str).context(InvalidConfiguration { path })?;

    Ok(conf)
}
