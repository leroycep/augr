use augr_core::{store::patch::CreateEvent, Patch};
use chrono::{DateTime, Utc};
use snafu::{ResultExt, Snafu};
use std::collections::{BTreeMap, BTreeSet};
use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to read timesheet from {}: {}", path.display(), source))]
    ReadTimesheet { source: io::Error, path: PathBuf },

    #[snafu(display("Error listing sync folder ({}) items: {}", path.display(), source))]
    SyncFolder { source: io::Error, path: PathBuf },

    #[snafu(display("{}:{} invalid datetime {}", path.display(), line_number, source))]
    DateTimeParse {
        source: chrono::format::ParseError,
        path: PathBuf,
        line_number: usize,
    },
}

pub fn import<P: AsRef<Path>>(sync_folder: P) -> Result<Vec<Patch>, Error> {
    let mut patches = Vec::new();

    let sync_folder = sync_folder.as_ref().to_path_buf();

    let sync_folder_items = sync_folder.read_dir().context(SyncFolder {
        path: sync_folder.clone(),
    })?;
    for dir_entry in sync_folder_items.filter_map(|d| d.ok()) {
        let path = dir_entry.path();
        if !path.is_file() {
            continue;
        }
        let timesheet = load_events(&path)?;

        let mut patch = Patch::new();
        for (start, tags) in timesheet {
            let event = uuid::Uuid::new_v4().to_string();
            patch.create_event.insert(CreateEvent {
                event,
                start,
                tags: tags.iter().cloned().collect(),
            });
        }

        patches.push(patch);
    }

    Ok(patches)
}

pub fn load_events(path: &Path) -> Result<BTreeMap<DateTime<Utc>, BTreeSet<String>>, Error> {
    let mut timesheet = BTreeMap::new();
    let contents = read_to_string(path).context(ReadTimesheet { path })?;

    if contents.trim() == "" {
        return Ok(timesheet);
    }

    for (line_number, line) in contents.lines().enumerate() {
        let mut cols = line.split(' ');
        let time = cols
            .next()
            .unwrap()
            .parse()
            .context(DateTimeParse { line_number, path })?;
        let tags = cols.map(|x| x.into()).collect();
        timesheet.insert(time, tags);
    }

    Ok(timesheet)
}
