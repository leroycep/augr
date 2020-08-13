use crate::config::Config;
use crate::{format_duration, time_input::parse_default_local};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Utc};
use clap::arg_enum;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use structopt::StructOpt;
use toml_edit::Document;

#[derive(StructOpt, Default, Debug)]
pub struct SummaryCmd {
    /// A list of tags to filter against
    tags: Vec<String>,

    /// Show the time that each event ended
    #[structopt(long = "show-ends")]
    show_ends: bool,

    /// Show event references as tags in output
    #[structopt(long = "refs")]
    show_refs: bool,

    /// The datetime at which to begin showing events
    #[structopt(long = "start", parse(try_from_os_str = parse_default_local))]
    start: Option<DateTime<Local>>,

    /// The datetime at which to stop showing events
    #[structopt(long = "end", parse(try_from_os_str = parse_default_local))]
    end: Option<DateTime<Local>>,

    /// How to treat the events on the edge of the start and end times
    ///
    /// "include": Keep the events that started before (but ended after) the start time.
    /// Same for the events that end after (but started before) the end time.
    ///
    /// "exclude": Only include events that are fully within the start and end times
    ///
    /// "clip": Keep the events at the start and end, but clip their durations to fit
    ///  within the start and end times.
    #[structopt(long = "edges", case_insensitive = true, default_value = "include")]
    edge_behavior: EdgeBehavior,
}

arg_enum! {
    #[derive(Debug)]
    pub enum EdgeBehavior {
        Include,
        Exclude,
        Clip,
    }
}

impl Default for EdgeBehavior {
    fn default() -> Self {
        EdgeBehavior::Include
    }
}

impl SummaryCmd {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn exec(&self, config: &Config) -> Result<()> {
        let filter_tags: BTreeSet<String> = self.tags.iter().cloned().collect();

        let start = self.start.unwrap_or_else(default_start);
        let end = self.end.unwrap_or_else(default_end);
        let mut segments: BTreeMap<DateTime<Local>, BTreeSet<String>> = BTreeMap::new();

        let walker = walkdir::WalkDir::new(&config.sync_folder)
            .into_iter()
            .filter_entry(|e| match e.depth() {
                0 => true,
                1 => match osstr_to_number(e.file_name()) {
                    Some(num) => num >= start.year(),
                    None => false,
                },
                2 => match osstr_to_number(e.file_name()) {
                    Some(num) => num >= start.month() as i32,
                    None => false,
                },
                3 => e.file_type().is_file(),
                _ => false,
            });

        for entry in walker {
            let entry = entry.context("Failed to read entry")?;
            if entry.depth() != 3 {
                continue;
            }
            let time_str = match entry.file_name().to_str() {
                Some(file_name) => file_name,
                None => continue,
            };
            let naive_time = NaiveDateTime::parse_from_str(time_str, "%Y%m%d-%H%M%S.toml")
                .with_context(|| {
                    format!("Failed to parse datetime from file path {:?}", entry.path())
                })?;

            let utc_time = DateTime::<Utc>::from_utc(naive_time, Utc);
            let local_time = utc_time.with_timezone(&Local);

            let record_contents = std::fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read record {:?}", entry.path()))?;

            let record_doc = record_contents
                .parse::<Document>()
                .with_context(|| format!("Invalid toml file {:?}", entry.path()))?;

            let tags_in_doc = match record_doc["tags"].as_array() {
                Some(array) => array,
                None => return Err(anyhow!("Expected tags to be an array")),
            };
            let mut tags = BTreeSet::new();

            for tag_in_doc in tags_in_doc.iter() {
                tags.insert(tag_in_doc.as_str().unwrap().into());
            }

            segments.insert(local_time, tags);
        }

        let first_time = segments
            .range(..start)
            .last()
            .map(|(time, _)| *time)
            .unwrap_or(start);
        let mut segments_iter = segments.range(first_time..).peekable();
        let mut total_duration = Duration::seconds(0);

        loop {
            let (&time, ref tags) = match segments_iter.next() {
                Some(a) => match self.edge_behavior {
                    EdgeBehavior::Clip => {
                        if *a.0 < start {
                            (&start, a.1)
                        } else {
                            a
                        }
                    }
                    EdgeBehavior::Include => a,
                    EdgeBehavior::Exclude => {
                        if *a.0 < start {
                            continue;
                        } else {
                            a
                        }
                    }
                },
                None => break,
            };

            if !tags.is_superset(&filter_tags) {
                continue;
            }

            if time > end {
                break;
            }

            let next_time_or_now = segments_iter
                .peek()
                .map(|(time, _)| *time)
                .cloned()
                .unwrap_or_else(Local::now);
            let next_time = match self.edge_behavior {
                EdgeBehavior::Clip => {
                    if next_time_or_now > end {
                        end
                    } else {
                        next_time_or_now
                    }
                }
                EdgeBehavior::Include => next_time_or_now,
                EdgeBehavior::Exclude => {
                    if next_time_or_now > end {
                        break;
                    } else {
                        next_time_or_now
                    }
                }
            };

            let duration = next_time.signed_duration_since(time);
            total_duration = total_duration + duration;

            println!(
                "{}\t{}\t{}\t{:?}",
                time.to_rfc3339(),
                format_duration(duration),
                format_duration(total_duration),
                tags
            );
        }

        Ok(())
    }
}

fn osstr_to_number(ostr: &OsStr) -> Option<i32> {
    let str = match ostr.to_str() {
        Some(s) => s,
        None => return None,
    };
    str.parse::<i32>().ok()
}

fn default_start() -> DateTime<Local> {
    Local::today().and_hms(0, 0, 0)
}

fn default_end() -> DateTime<Local> {
    Local::now()
}
