use crate::{
    database::DataBase,
    timesheet::{load_transitions, save_timesheet, Tag, Timesheet},
};
use chrono::{DateTime, Utc};
use snafu::{ResultExt, Snafu};
use std::collections::{BTreeMap, BTreeSet};
use std::{
    io::Error as IOError,
    path::{Path, PathBuf},
};

pub struct SyncFolderDB {
    sync_folder: PathBuf,
    device_path: PathBuf,
    device_timesheet: Timesheet,
    global_timesheet: Timesheet,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to get contents of sync folder {}: {}", path.display(), source))]
    SyncFolder { source: IOError, path: PathBuf },

    #[snafu(display("Unable to read file {}: {}", path.display(), source))]
    ReadFile {
        source: crate::timesheet::Error,
        path: PathBuf,
    },

    #[snafu(display("Unable to write file {}: {}", path.display(), source))]
    WriteFile {
        source: crate::timesheet::Error,
        path: PathBuf,
    },
}

impl SyncFolderDB {
    pub fn load(sync_folder: &Path, device_id: String) -> Result<SyncFolderDB, Error> {
        let device_path = sync_folder.join(device_id).with_extension("unknown");
        let sync_folder = sync_folder.to_path_buf();
        let mut device_timesheet = Timesheet::new();

        load_transitions(&device_path, &mut device_timesheet).context(ReadFile {
            path: device_path.to_path_buf(),
        })?;

        let mut db = SyncFolderDB {
            sync_folder,
            device_path,
            global_timesheet: device_timesheet.clone(),
            device_timesheet,
        };

        let sync_folder_items = db.sync_folder.read_dir().context(SyncFolder {
            path: db.sync_folder.clone(),
        })?;
        for dir_entry in sync_folder_items.filter_map(|d| d.ok()) {
            let path = dir_entry.path();
            if path == db.device_path {
                continue;
            }
            if !path.is_file() {
                continue;
            }
            load_transitions(&path, &mut db.global_timesheet).context(ReadFile { path })?;
        }

        Ok(db)
    }

    pub fn save(&self) -> Result<(), Error> {
        save_timesheet(&self.device_path, &self.device_timesheet).context(WriteFile {
            path: &self.device_path.to_path_buf(),
        })?;
        Ok(())
    }
}

impl DataBase for SyncFolderDB {
    fn transitions(&self) -> BTreeMap<&DateTime<Utc>, &BTreeSet<Tag>> {
        self.global_timesheet.transitions()
    }

    fn insert_transition(&mut self, datetime: DateTime<Utc>, tags: BTreeSet<Tag>) {
        self.device_timesheet
            .insert_transition(datetime.clone(), tags.clone());
        self.global_timesheet.insert_transition(datetime, tags);
    }
}
