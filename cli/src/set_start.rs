use augr_core::{
    store::patch::{AddStart, RemoveStart},
    EventRef, Patch, Timesheet,
};
use chrono::{DateTime, Local, Utc};
use snafu::Snafu;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// The id of the event to modify
    event: EventRef,

    /// The time when you started
    #[structopt(parse(try_from_os_str = crate::time_input::parse_default_local))]
    time: DateTime<Local>,
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
        for (patch_ref, previous_start_time) in event.starts() {
            patch.insert_remove_start(RemoveStart {
                parents: Some(parent_patches.clone()),
                event: self.event.clone(),
                patch: patch_ref,
                time: previous_start_time,
            });
        }
        patch.insert_add_start(AddStart {
            parents: parent_patches.clone(),
            event: self.event.clone(),
            time: self.time.with_timezone(&Utc),
        });
        Ok(vec![patch])
    }
}
