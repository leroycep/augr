use crate::{config::Config, summary::get_segments};
use anyhow::Result;
use chrono::Local;
use std::{collections::BTreeMap, io::Write};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "tags")]
pub struct Cmd {}

impl Cmd {
    pub fn exec(&self, config: &Config) -> Result<()> {
        let mut tags = BTreeMap::new();

        let segments = get_segments(config, &Local)?;

        for (_time, segment_tags) in segments.iter() {
            for tag in segment_tags.iter() {
                *tags.entry(tag).or_insert(0) += 1;
            }
        }

        let mut tags_sorted_by_appearances: Vec<_> = tags.iter().collect();
        tags_sorted_by_appearances.sort_by_key(|(_tag, num_appearances)| -*num_appearances);

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();

        for (tag, num_appearances) in tags_sorted_by_appearances {
            match writeln!(stdout, "{} {}", num_appearances, tag) {
                Ok(()) => {}

                // May mean that a command like `head` was used, stop outputting
                Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => break,

                Err(e) => return Err(e.into()),
            };
        }

        Ok(())
    }
}
