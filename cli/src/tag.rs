use augr_core::{store::patch::AddTag, EventRef, Patch, Timesheet};
use snafu::Snafu;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// The id of the event to modify
    event: EventRef,

    /// A list of tags to append to the event
    #[structopt(raw(required = "true"))]
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
        for tag in self.tags.iter().cloned() {
            patch.insert_add_tag(AddTag {
                parents: parent_patches.clone(),
                event: self.event.clone(),
                tag,
            });
        }
        Ok(vec![patch])
    }
}
