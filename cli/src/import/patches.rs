use anyhow::{anyhow, Context};
use augr_core::{store::SyncFolderStore, Repository};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use toml_edit::Document;

pub fn import<P: AsRef<Path>>(
    current_config: &crate::config::Config,
    old_config_path: P,
) -> anyhow::Result<()> {
    let old_conf = load_config(old_config_path.as_ref()).context("Could not load config file")?;

    let store = SyncFolderStore::new(old_conf.sync_folder, old_conf.device_id).should_init(true);
    let mut repo = Repository::from_store(store).unwrap();

    match repo.try_sync_data() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Failed to sync data in old format: {:?}", e);
            eprintln!("Continue? (y/N)");
            todo!();
        }
    };

    let eventgraph = repo.timesheet();
    let timesheet = eventgraph
        .flatten()
        .map_err(|conflicts| anyhow!("Merging conflict in old format: {:?}", conflicts))?;

    let segments = timesheet.segments().into_iter();

    for segment in segments {
        let tags_vec = segment.tags.into_iter().collect::<Vec<_>>();
        if let Err(e) = crate::start::add_event(current_config, segment.start_time, &tags_vec) {
            eprintln!("Error importing data, ignoring: {}", e);
        }
    }

    Ok(())
}

pub struct Config {
    pub sync_folder: PathBuf,
    pub device_id: String,
}

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let conf_str = read_to_string(path)
        .with_context(|| format!("Failed to read configuration at {:?}", path))?;

    let conf_doc = conf_str
        .parse::<Document>()
        .context("Invalid configuration file")?;

    let sync_folder = match conf_doc["sync_folder"].as_str() {
        Some(toml_value) => toml_value.into(),
        None => return Err(anyhow!("sync_folder is not defined in config file")),
    };
    let device_id = match conf_doc["device_id"].as_str() {
        Some(toml_value) => toml_value.into(),
        None => return Err(anyhow!("device_id is not defined in config file")),
    };

    Ok(Config {
        sync_folder,
        device_id,
    })
}
