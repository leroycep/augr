use augr_core::{store::SyncFolderStore, Meta, Patch, Repository, Store, Tag};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

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
                temp_meta.add_patch($x.clone());
            )*
            temp_meta
        }
    };
}

fn simple_store() -> SyncFolderStore {
    SyncFolderStore::new("tests/basic_repo".into(), "laptop".into())
}

#[test]
fn load_patches_into_store() {
    let patch1 = &Uuid::parse_str("d83f2984-8f59-4a32-9492-f910717b683c").unwrap();
    let patch2 = &Uuid::parse_str("386d2d62-7c3f-4518-9709-d2145261b853").unwrap();

    let expected_meta = meta![patch2.clone()];
    let expected_patches = vec![
        Patch::with_id(patch1.clone())
            .create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"])
            .create_event(s!("b"), dt!("2019-07-23T13:00:00Z"), sl!["work"]),
        Patch::with_id(patch2.clone())
            .remove_start(patch1.clone(), s!("a"), dt!("2019-07-23T12:00:00Z"))
            .add_start(patch1.clone(), s!("a"), dt!("2019-07-23T12:30:00Z"))
            .remove_tag(patch1.clone(), s!("a"), s!("food"))
            .add_tag(patch1.clone(), s!("b"), s!("awesome-project")),
    ];

    let store = simple_store();

    assert_eq!(store.get_meta().unwrap(), expected_meta);
    for patch in expected_patches {
        assert_eq!(store.get_patch(patch.patch_ref()).unwrap(), patch);
    }
}

#[test]
fn check_repository_state() {
    let repository = dbg!(Repository::from_store(simple_store()));
    assert!(repository.is_ok());
    let repository = repository.unwrap();

    let current_timesheet = repository.timesheet();

    let mut expected_timesheet: BTreeMap<DateTime<Utc>, BTreeSet<Tag>> = BTreeMap::new();
    expected_timesheet.insert(dt!("2019-07-23T12:30:00Z"), sl!["lunch"]);
    expected_timesheet.insert(dt!("2019-07-23T13:00:00Z"), sl!["work", "awesome-project"]);

    let timesheet = current_timesheet.flatten();
    assert!(timesheet.is_ok());
    assert!(timesheet.unwrap().eq(&expected_timesheet));
}
