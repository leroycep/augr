use chrono::{DateTime, Duration, Utc};
use snafu::{ResultExt, Snafu};
use std::collections::{BTreeMap, HashSet};
use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Timesheet {
    transitions: BTreeMap<DateTime<Utc>, HashSet<Tag>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Tag(pub String);

#[derive(Clone, Debug)]
pub struct Segment {
    pub start_time: DateTime<Utc>,
    pub tags: HashSet<Tag>,
    pub duration: Duration,
    pub end_time: DateTime<Utc>,
}

impl Timesheet {
    pub fn new() -> Timesheet {
        Self {
            transitions: BTreeMap::new(),
        }
    }

    pub fn transitions<'ts>(&'ts self) -> impl Iterator<Item = (&DateTime<Utc>, &HashSet<Tag>)> {
        self.transitions.iter()
    }

    pub fn segments(&self) -> Vec<Segment> {
        let now = Utc::now();
        let end_cap_arr = [now];
        self.transitions
            .iter()
            .zip(self.transitions.keys().skip(1).chain(end_cap_arr.iter()))
            .map(|(t, end_time)| {
                let duration = end_time.signed_duration_since(*t.0);
                Segment {
                    start_time: t.0.clone(),
                    tags: t.1.clone(),
                    duration,
                    end_time: end_time.clone(),
                }
            })
            .collect()
    }

    pub fn insert_transition(
        &mut self,
        datetime: DateTime<Utc>,
        tags: HashSet<Tag>,
    ) -> Option<HashSet<Tag>> {
        self.transitions.insert(datetime, tags)
    }

    pub fn tags_at_time<'ts>(&'ts self, datetime: &DateTime<Utc>) -> Option<&'ts HashSet<Tag>> {
        self.transitions
            .range(..datetime)
            .map(|(_time, tags)| tags)
            .last()
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

pub fn load_timesheet(path: &Path) -> Result<Timesheet, Error> {
    let mut timesheet = Timesheet::new();
    if !path.exists() {
        return Ok(timesheet);
    }

    let contents = read_to_string(path).unwrap();

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
    Ok(timesheet)
}

pub fn save_timesheet(path: &Path, timesheet: &Timesheet) -> Result<(), Error> {
    use std::io::Write;

    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut wtr = std::fs::OpenOptions::new()
        .write(true)
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
