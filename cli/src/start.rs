use augr_core::{Patch, Timesheet};
use chrono::{DateTime, Local, Utc};
use structopt::StructOpt;
use std::path::PathBuf;
use toml_edit::{Document, value, Array};

#[derive(StructOpt, Debug)]
pub struct StartCmd {
    /// The time when you started
    #[structopt(long = "time", parse(try_from_os_str = crate::time_input::parse_default_local))]
    time: Option<DateTime<Local>>,

    /// A list of tags showing what you are doing
    tags: Vec<String>,
}

impl StartCmd {
    pub fn exec(&self, _timesheet: &Timesheet) -> Vec<Patch> {
        let now = Local::now();
        let start_time = self
            .time
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);
        let tags = self.tags.to_vec();

        let year_repr = start_time.format("%Y");
        let month_repr = start_time.format("%m");
        let filename_repr = start_time.format("%Y%m%d-%H%M%S");

        let mut filepath = PathBuf::new();
        filepath.push("target");
        filepath.push("testing_folder");
        filepath.push(format!("{}", year_repr));
        filepath.push(format!("{}", month_repr));

        std::fs::create_dir_all(&filepath).unwrap();

        filepath.push(format!("{}", filename_repr));
        filepath.set_extension("toml");

        if filepath.exists() {
            eprintln!("A record already exists at {}", start_time);
            std::process::exit(1);
        }

        let mut doc = Document::new();

        doc["created"] = value(now.to_rfc3339());

        let mut tags_array = Array::default();
        for tag in tags {
            tags_array.push(tag).unwrap();
        }
        doc["tags"] = value(tags_array);

        let contents = doc.to_string();

        std::fs::write(filepath, &contents).unwrap();

        vec![]
    }
}
