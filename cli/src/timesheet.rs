use crate::database::DataBase;
use chrono::{DateTime, Duration, Utc};
use snafu::{ResultExt, Snafu};
use std::collections::{BTreeMap, BTreeSet};
use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug)]
pub struct Timesheet {
    transitions: BTreeMap<DateTime<Utc>, BTreeSet<Tag>>,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Tag(pub String);

#[derive(Clone, Debug)]
pub struct Segment {
    pub start_time: DateTime<Utc>,
    pub tags: BTreeSet<Tag>,
    pub duration: Duration,
    pub end_time: DateTime<Utc>,
}

impl Timesheet {
    pub fn new() -> Timesheet {
        Self {
            transitions: BTreeMap::new(),
        }
    }
}

impl DataBase for Timesheet {
    fn transitions(&self) -> BTreeMap<&DateTime<Utc>, &BTreeSet<Tag>> {
        self.transitions.iter().collect()
    }

    fn insert_transition(&mut self, datetime: DateTime<Utc>, tags: BTreeSet<Tag>) {
        self.transitions.insert(datetime, tags);
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to read timesheet from {}: {}", path.display(), source))]
    ReadTimesheet { source: io::Error, path: PathBuf },

    #[snafu(display("Unable to write timesheet to {}: {}", path.display(), source))]
    WriteTimesheet { source: io::Error, path: PathBuf },

    #[snafu(display("{}:{} invalid datetime {}", path.display(), line_number, source))]
    DateTimeParse {
        source: chrono::format::ParseError,
        path: PathBuf,
        line_number: usize,
    },
}

pub fn load_transitions(path: &Path, timesheet: &mut Timesheet) -> Result<(), Error> {
    let contents = read_to_string(path).context(ReadTimesheet { path })?;

    if contents.trim() == "" {
        return Ok(());
    }

    for (line_number, line) in contents.lines().enumerate() {
        let mut cols = line.split(' ');
        let time = cols
            .next()
            .unwrap()
            .parse()
            .context(DateTimeParse { line_number, path })?;
        let tags = cols.map(|x| Tag(x.into())).collect();
        timesheet.transitions.insert(time, tags);
    }

    Ok(())
}

pub fn save_timesheet(path: &Path, timesheet: &Timesheet) -> Result<(), Error> {
    use std::io::Write;

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut wtr = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .context(WriteTimesheet { path })?;

    for (start_time, tags) in timesheet.transitions.iter() {
        write!(wtr, "{}", start_time.to_rfc3339()).unwrap();
        for t in tags {
            write!(wtr, " {}", t.0).unwrap();
        }
        wtr.write(b"\n").unwrap();
    }
    wtr.flush().unwrap();

    Ok(())
}
