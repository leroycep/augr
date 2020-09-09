use crate::config::Config;
use crate::{format_duration, time_input::parse_default_local};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Utc};
use clap::arg_enum;
use prettytable::{format::FormatBuilder, Table};
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

    #[structopt(long = "format", case_insensitive = true, default_value = "ascii")]
    output_format: Format,
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

arg_enum! {
    #[derive(Debug)]
    pub enum Format {
        Ascii,
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Ascii
    }
}

impl SummaryCmd {
    #[cfg_attr(feature = "flame_it", flame)]
    pub fn exec(&self, config: &Config) -> Result<()> {
        if !config.sync_folder.exists() {
            return Err(anyhow!("Sync folder does not exist"));
        }

        let segments = get_segments(config, &Local)?;

        let filter_tags: BTreeSet<String> = self.tags.iter().cloned().collect();

        let now = Local::now();
        let start = self.start.unwrap_or_else(default_start);
        let end = self.end.unwrap_or_else(default_end);
        let first_time = segments
            .range(..start)
            .last()
            .map(|(time, _)| *time)
            .unwrap_or(start);
        let mut segments_iter = segments.range(first_time..).peekable();
        let mut total_duration = Duration::seconds(0);

        let mut table = Table::new();

        let table_format = FormatBuilder::new().padding(0, 1).build();
        table.set_format(table_format);

        table.set_titles(row![
            u -> "Wk", u -> "Date", u -> "Day", u -> "Tags", ur -> "Start", ur -> "End", ur ->"Time", ur -> "Total"
        ]);

        let mut prev_date_opt: Option<chrono::Date<Local>> = None;

        'SEGMENTS_LOOP: loop {
            let (&time, ref tags, first_time_in_range) = match segments_iter.next() {
                Some(a) => match self.edge_behavior {
                    EdgeBehavior::Clip => {
                        if *a.0 < start {
                            (&start, a.1, false)
                        } else {
                            (a.0, a.1, true)
                        }
                    }
                    EdgeBehavior::Include => (a.0, a.1, true),
                    EdgeBehavior::Exclude => {
                        if *a.0 < start {
                            continue;
                        } else {
                            (a.0, a.1, true)
                        }
                    }
                },
                None => break,
            };

            // Check that tags has all the filter tags that were specified
            for filter_tag in &filter_tags {
                if !tags.contains(filter_tag) {
                    continue 'SEGMENTS_LOOP;
                }
            }

            if time > end {
                break;
            }

            let (next_time_or_now, was_next_time) = segments_iter
                .peek()
                .cloned()
                .map(|(time, _)| (*time, true))
                .unwrap_or_else(|| (now, false));
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

            // Initialize wk, date, and day to current date
            let mut wk_text = time.format("W%W");
            let mut date_text = time.format("%Y-%m-%d");
            let mut day_text = time.format("%a");

            if let Some(prev_date) = prev_date_opt {
                if time.iso_week() == prev_date.iso_week() {
                    wk_text = time.format("");
                }
                if time.date() == prev_date {
                    date_text = time.format("");
                    day_text = time.format("");
                }
            };
            prev_date_opt = Some(time.date());

            let start_time_text = if first_time_in_range {
                time.format("%H:%M").to_string()
            } else {
                String::from("--")
            };

            let end_time_text = if was_next_time {
                next_time.format("%H:%M").to_string()
            } else {
                String::from("--")
            };

            match self.output_format {
                Format::Ascii => table.add_row(row!(
                    wk_text,
                    date_text,
                    day_text,
                    tags.join(" "),
                    r -> start_time_text,
                    r -> end_time_text,
                    r -> format_duration(duration),
                    r -> format_duration(total_duration),
                )),
            };
        }

        let total_duration_text = format_duration(total_duration);
        let blank_space_size_of_duration = total_duration_text
            .chars()
            .map(|_c| ' ')
            .collect::<String>();
        table.add_row(row!("", "", "", "", "", "", "", ur -> blank_space_size_of_duration));
        table.add_row(row!("", "", "", "", "", "", "", r -> total_duration_text));

        table.printstd();

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

//struct Tags {
//    // A list of all the tags used
//    tags: Vec<String>,
//    segments: BTreeMap<DateTime<TZ>, BTreeSet<usize>>
//}

pub fn get_segments<TZ: chrono::offset::TimeZone>(
    config: &Config,
    tz: &TZ,
) -> Result<BTreeMap<DateTime<TZ>, Vec<String>>> {
    let mut segments: BTreeMap<DateTime<TZ>, Vec<String>> = BTreeMap::new();

    let walker = walkdir::WalkDir::new(&config.sync_folder)
        .into_iter()
        .filter_entry(|e| match e.depth() {
            0 => true,
            1 => osstr_to_number(e.file_name()).is_some(),
            2 => osstr_to_number(e.file_name()).is_some(),
            3 => e.file_type().is_file(),
            _ => false,
        });

    for entry in walker {
        let entry = entry.context("Failed to read entry")?;
        if entry.depth() != 3 {
            continue;
        }

        let time = {
            let time_str = match entry.file_name().to_str() {
                Some(file_name) => file_name,
                None => continue,
            };
            let naive_time = NaiveDateTime::parse_from_str(time_str, "%Y%m%d-%H%M%S.toml")
                .with_context(|| {
                    format!("Failed to parse datetime from file path {:?}", entry.path())
                })?;

            DateTime::<Utc>::from_utc(naive_time, Utc).with_timezone(tz)
        };

        let record_contents = std::fs::read_to_string(entry.path())
            .with_context(|| format!("Failed to read record {:?}", entry.path()))?;

        let record_doc = record_contents
            .parse::<Document>()
            .with_context(|| format!("Invalid toml file {:?}", entry.path()))?;

        let tags_in_doc = match record_doc["tags"].as_array() {
            Some(array) => array,
            None => return Err(anyhow!("Expected tags to be an array")),
        };
        let mut tags = Vec::new();

        for tag_in_doc in tags_in_doc.iter() {
            tags.push(tag_in_doc.as_str().unwrap().into());
        }

        segments.insert(time, tags);
    }

    Ok(segments)
}
