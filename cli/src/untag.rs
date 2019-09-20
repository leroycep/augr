use augr_core::{store::patch::RemoveTag, EventRef, Patch, Timesheet};
use snafu::Snafu;
use std::collections::BTreeSet;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// The id of the event to modify
    event: EventRef,

    /// A list of tags to remove from the event
    #[structopt(required = true)]
    tags: Vec<String>,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unknown event reference: {}", event_ref))]
    UnknownEventRef { event_ref: EventRef },
    #[snafu(display("Event does not contain the following tags: {:?}", tags))]
    UnknownTags { tags: BTreeSet<String> },
}

impl Cmd {
    pub fn exec(&self, timesheet: &Timesheet) -> Result<Vec<Patch>, Error> {
        let event = timesheet
            .get_patched_timesheet()
            .events
            .get(&self.event)
            .ok_or(Error::UnknownEventRef {
                event_ref: self.event.clone(),
            })?;
        let parent_patches = event.latest_patches();
        let mut patch = Patch::new();
        let mut tags_to_remove: BTreeSet<_> = self.tags.iter().collect();
        for (patch_ref_added, tag) in event.tags() {
            if tags_to_remove.contains(&tag) {
                tags_to_remove.remove(&tag);
                let mut parents = parent_patches.clone();
                parents.remove(&patch_ref_added);
                let parents = if parents.is_empty() {
                    None
                } else {
                    Some(parents)
                };

                patch.insert_remove_tag(RemoveTag {
                    parents,
                    patch: patch_ref_added,
                    event: self.event.clone(),
                    tag,
                });
            }
        }
        // If there are any tags left that haven't been removed, the tag doesn't exist
        // on the event
        // TODO: figure out how to test this
        if !tags_to_remove.is_empty() {
            Err(Error::UnknownTags {
                tags: tags_to_remove.into_iter().cloned().collect(),
            })
        } else {
            Ok(vec![patch])
        }
    }
}
