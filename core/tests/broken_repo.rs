use augr_core::{
    repository::{
        event::Error as EventError, timesheet::Error as TimesheetError, Error as RepositoryError,
    },
    Meta, Patch, PatchRef, Repository, Store,
};
use chrono::{DateTime, Utc};
use snafu::Snafu;
use std::collections::BTreeMap;

#[derive(Debug)]
struct MemStore {
    meta: Meta,
    patches: BTreeMap<String, Patch>,
}

impl MemStore {
    pub fn new(meta: Meta) -> Self {
        Self {
            meta,
            patches: BTreeMap::new(),
        }
    }

    pub fn patch(mut self, patch_ref: &str, patch: Patch) -> Self {
        self.patches.insert(patch_ref.to_string(), patch);
        self
    }
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum MemStoreError {
    #[snafu(display("Meta not found {}", device_id))]
    MetaNotFound { device_id: String },

    #[snafu(display("Patch not found {}", patch_ref))]
    PatchNotFound { patch_ref: String },
}

impl Store for MemStore {
    type Error = MemStoreError;

    fn get_meta(&self) -> Result<Meta, Self::Error> {
        Ok(self.meta.clone())
    }

    fn save_meta(&mut self, _meta: &Meta) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn add_patch(&mut self, _patch: &Patch) -> Result<PatchRef, Self::Error> {
        unimplemented!()
    }

    fn get_patch(&self, patch_ref: &str) -> Result<Patch, Self::Error> {
        self.patches
            .get(patch_ref)
            .map(|x| x.clone())
            .ok_or(MemStoreError::PatchNotFound {
                patch_ref: patch_ref.to_string(),
            })
    }
}

macro_rules! dt {
    ( $dt:expr ) => {{
        $dt.parse::<DateTime<Utc>>().expect("Valid datetime")
    }};
}

macro_rules! sl {
    ( $( $s:expr ),* ) => {
        [ $( $s, )* ].into_iter().map(|sv| sv.to_string() ).collect()
    };
}

macro_rules! s {
    ($s:expr) => {
        $s.to_string()
    };
}

macro_rules! meta {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_meta = Meta::new();
            $(
                temp_meta.add_patch($x.to_string());
            )*
            temp_meta
        }
    };
}

#[test]
fn unknown_event_ref_reported() {
    let store = MemStore::new(meta!["2"])
        .patch(
            "1",
            Patch::new().create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"]),
        )
        .patch(
            "2",
            Patch::new().remove_start(s!("1"), s!("b"), dt!("2019-07-23T12:00:00Z")),
        );

    let errors = Repository::from_store(store).expect_err("patches to produce error");

    assert!(errors.contains(&RepositoryError::PatchingTimesheet {
        patch: s!("2"),
        conflicts: vec![TimesheetError::UnknownEvent {
            patch: s!("2"),
            event: s!("b")
        }]
    }));
}

#[test]
fn unknown_patch_reported() {
    let store = MemStore::new(meta!["2"]).patch("1", Patch::new());

    let errors = Repository::from_store(store).unwrap_err();

    assert!(errors.contains(&RepositoryError::PatchNotFound {
        source: MemStoreError::PatchNotFound { patch_ref: s!("2") },
        patch: s!("2")
    }));
}

#[test]
fn invalid_number_of_start_times() {
    let store = MemStore::new(meta!["2", "3"])
        .patch(
            "1",
            Patch::new()
                .create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"])
                .create_event(s!("b"), dt!("2019-07-23T13:00:00Z"), sl!["work"]),
        )
        .patch(
            "2",
            Patch::new().add_start(s!("1"), s!("a"), dt!("2019-07-23T12:30:00Z")),
        )
        .patch(
            "3",
            Patch::new().remove_start(s!("1"), s!("b"), dt!("2019-07-23T13:00:00Z")),
        );

    let repo = Repository::from_store(store).unwrap();
    let current_timesheet = repo.timesheet();
    let errors = current_timesheet
        .flatten()
        .expect_err("flattening conflicted repository to report errors");

    assert!(errors.contains(&TimesheetError::FlattenEventError {
        source: EventError::MultipleStartTimes,
        event: s!("a")
    }));
    assert!(errors.contains(&TimesheetError::FlattenEventError {
        source: EventError::NoStartTimes,
        event: s!("b")
    }));
}
