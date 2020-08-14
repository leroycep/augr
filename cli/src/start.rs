use crate::config::Config;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Local, Utc};
use structopt::StructOpt;
use toml_edit::{value, Array, Document};

#[derive(StructOpt, Debug)]
pub struct StartCmd {
    /// The time when you started
    #[structopt(long = "time", parse(try_from_os_str = crate::time_input::parse_default_local))]
    time: Option<DateTime<Local>>,

    /// A list of tags showing what you are doing
    tags: Vec<String>,
}

impl StartCmd {
    pub fn exec(&self, config: &Config) -> anyhow::Result<()> {
        if !config.sync_folder.exists() {
            eprintln!("Sync folder does not exist; creating folder at {:?}", config.sync_folder);
        }

        let now = Local::now();
        let start_time = self
            .time
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let tags = self.tags.to_vec();

        let year_repr = start_time.format("%Y");
        let month_repr = start_time.format("%m");
        let filename_repr = start_time.format("%Y%m%d-%H%M%S");

        let mut filepath = config.sync_folder.clone();
        filepath.push(format!("{}", year_repr));
        filepath.push(format!("{}", month_repr));

        std::fs::create_dir_all(&filepath).with_context(|| {
            format!("Failed to create records directory {}", filepath.display())
        })?;

        filepath.push(format!("{}", filename_repr));
        filepath.set_extension("toml");

        if filepath.exists() {
            return Err(anyhow!("A record already exists for {} ({})", start_time.with_timezone(&Local), start_time));
        }

        let mut doc = Document::new();

        doc["created"] = value(now.to_rfc3339());

        let mut tags_array = Array::default();
        for tag in tags {
            tags_array.push(tag).unwrap();
        }
        doc["tags"] = value(tags_array);

        let contents = doc.to_string();

        std::fs::write(&filepath, &contents)
            .with_context(|| format!("Failed to write record to {}", filepath.display()))?;

        Ok(())
    }
}
