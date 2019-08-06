use augr_core::{
    repository::{
        event::Error as EventError, timesheet::Error as TimesheetError, Error as RepositoryError,
    },
    Event, Meta, Patch, Repository, Store, Timesheet,
};
use chrono::{DateTime, Utc};
use snafu::{ResultExt, Snafu};
use std::collections::BTreeMap;

struct MemStore {
    metas: BTreeMap<String, Meta>,
    patches: BTreeMap<String, Patch>,
}

impl MemStore {
    pub fn new() -> Self {
        Self {
            metas: BTreeMap::new(),
            patches: BTreeMap::new(),
        }
    }

    pub fn meta(mut self, device_id: &str, meta: Meta) -> Self {
        self.metas.insert(device_id.to_string(), meta);
        self
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

    fn get_device_meta(&self, device_id: &str) -> Result<Meta, Self::Error> {
        self.metas
            .get(device_id)
            .map(|x| x.clone())
            .ok_or(MemStoreError::MetaNotFound {
                device_id: device_id.to_string(),
            })
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
    let store = MemStore::new()
        .meta("laptop", meta!["2"])
        .patch(
            "1",
            Patch::new().create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"]),
        )
        .patch(
            "2",
            Patch::new().remove_start(s!("1"), s!("b"), dt!("2019-07-23T12:00:00Z")),
        );

    let repo = Repository::from_store(store, s!("laptop"));
    let errors = dbg!(repo
        .get_current_timesheet()
        .expect_err("patches to produce error"));

    assert!(errors.contains(&RepositoryError::EventNotFound {
        patch: s!("2"),
        event: s!("b")
    }));
}

#[test]
fn unknown_patch_reported() {
    let store = MemStore::new()
        .meta("laptop", meta!["2"])
        .patch("1", Patch::new());

    let repo = Repository::from_store(store, s!("laptop"));
    let errors = repo
        .get_current_timesheet()
        .expect_err("unkown patch to be reported");

    assert!(errors.contains(&RepositoryError::PatchNotFound {
        source: MemStoreError::PatchNotFound { patch_ref: s!("2") },
        patch: s!("2")
    }));
}

#[test]
fn invalid_number_of_start_times() {
    let store = MemStore::new()
        .meta("laptop", meta!["2", "3"])
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

    let repo = Repository::from_store(store, s!("laptop"));
    let current_timesheet = dbg!(repo
        .get_current_timesheet()
        .expect("patches to build pathed timesheet"));
    let errors = dbg!(current_timesheet
        .flatten()
        .expect_err("flattening conflicted repository to report errors"));

    assert!(errors.contains(&TimesheetError::FlattenEventError {
        source: EventError::MultipleStartTimes,
        event: s!("a")
    }));
    assert!(errors.contains(&TimesheetError::FlattenEventError {
        source: EventError::NoStartTimes,
        event: s!("b")
    }));
}
