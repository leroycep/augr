use augr_core::{
    repository::{
        event::Error as EventError, timesheet::Error as TimesheetError, Error as RepositoryError,
    },
    Meta, Patch, PatchRef, Repository, Store,
};
use chrono::{DateTime, Utc};
use snafu::Snafu;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug)]
struct MemStore {
    meta: Meta,
    patches: BTreeMap<PatchRef, Patch>,
}

impl MemStore {
    pub fn new(meta: Meta) -> Self {
        Self {
            meta,
            patches: BTreeMap::new(),
        }
    }

    pub fn patch(mut self, patch: Patch) -> Self {
        self.patches.insert(patch.patch_ref().clone(), patch);
        self
    }
}

#[derive(Eq, PartialEq, Debug, Snafu)]
pub enum MemStoreError {
    #[snafu(display("Meta not found {}", device_id))]
    MetaNotFound { device_id: String },

    #[snafu(display("Patch not found {}", patch_ref))]
    PatchNotFound { patch_ref: PatchRef },
}

impl Store for MemStore {
    type Error = MemStoreError;

    fn get_meta(&self) -> Result<Meta, Self::Error> {
        Ok(self.meta.clone())
    }

    fn save_meta(&mut self, _meta: &Meta) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn add_patch(&mut self, _patch: &Patch) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn get_patch(&self, patch_ref: &PatchRef) -> Result<Patch, Self::Error> {
        self.patches
            .get(patch_ref)
            .map(|x| x.clone())
            .ok_or(MemStoreError::PatchNotFound {
                patch_ref: patch_ref.clone(),
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

macro_rules! p {
    ($s:expr) => {
        Patch::with_id($s.clone())
    };
}

macro_rules! meta {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_meta = Meta::new();
            $(
                temp_meta.add_patch($x.clone());
            )*
            temp_meta
        }
    };
}

#[test]
fn unknown_event_ref_reported() {
    let patch1 = &Uuid::parse_str("2a226f4d-60f2-493d-9e9a-d6c71d98b515").unwrap();
    let patch2 = &Uuid::parse_str("dad9051e-2e83-446e-b9aa-299bd4a34b37").unwrap();

    let store = MemStore::new(meta![patch2])
        .patch(p!(patch1).create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"]))
        .patch(p!(patch2).remove_start(patch1.clone(), s!("b"), dt!("2019-07-23T12:00:00Z")));

    let errors = Repository::from_store(store).expect_err("patches to produce error");

    assert!(errors.contains(&RepositoryError::PatchingTimesheet {
        patch: patch2.clone(),
        conflicts: vec![TimesheetError::UnknownEvent {
            patch: patch2.clone(),
            event: s!("b")
        }]
    }));
}

#[test]
fn unknown_patch_reported() {
    let patch1 = &Uuid::new_v4();
    let patch2 = &Uuid::new_v4();

    let store = MemStore::new(meta![patch2.clone()]).patch(p!(patch1));

    let errors = Repository::from_store(store).unwrap_err();

    assert!(errors.contains(&RepositoryError::PatchNotFound {
        source: MemStoreError::PatchNotFound {
            patch_ref: patch2.clone(),
        },
        patch: patch2.clone(),
    }));
}

#[test]
fn invalid_number_of_start_times() {
    let patch1 = &Uuid::new_v4();
    let patch2 = &Uuid::new_v4();
    let patch3 = &Uuid::new_v4();

    let store = MemStore::new(meta![patch2, patch3])
        .patch(
            p!(patch1)
                .create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"])
                .create_event(s!("b"), dt!("2019-07-23T13:00:00Z"), sl!["work"]),
        )
        .patch(p!(patch2).add_start(patch1.clone(), s!("a"), dt!("2019-07-23T12:30:00Z")))
        .patch(p!(patch3).remove_start(patch1.clone(), s!("b"), dt!("2019-07-23T13:00:00Z")));

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
