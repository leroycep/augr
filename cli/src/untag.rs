use augr_core::{store::patch::RemoveTag, EventRef, Patch, Timesheet};
use snafu::Snafu;
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
        for (patch_ref_added, tag) in event.tags() {
            if self.tags.contains(&tag) {
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
        Ok(vec![patch])
    }
}
