mod meta;
mod patch;
mod repository;
mod store;
mod timesheet;

pub use crate::meta::Meta;
pub use crate::patch::{Patch, PatchRef};
pub use crate::repository::Repository;
pub use crate::store::Store;
pub use crate::timesheet::{Event, Timesheet};

pub type EventRef = String;
pub type Tag = String;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
