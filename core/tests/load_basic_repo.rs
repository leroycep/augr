use augr_core::{store::SyncFolderStore, Event, Meta, Patch, Repository, Store, Timesheet};
use chrono::{DateTime, Utc};

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

fn simple_store() -> SyncFolderStore {
    SyncFolderStore::new("tests/basic_repo".into(), "laptop".into())
}

#[test]
fn load_patches_into_store() {
    let expected_metas = vec![("laptop", meta!["laptop-patch-2"])];
    let expected_patches = vec![
        (
            "laptop-patch-1",
            Patch::new()
                .create_event(s!("a"), dt!("2019-07-23T12:00:00Z"), sl!["lunch", "food"])
                .create_event(s!("b"), dt!("2019-07-23T13:00:00Z"), sl!["work"]),
        ),
        (
            "laptop-patch-2",
            Patch::new()
                .remove_start(s!("laptop-patch-1"), s!("a"), dt!("2019-07-23T12:00:00Z"))
                .add_start(s!("laptop-patch-1"), s!("a"), dt!("2019-07-23T12:30:00Z"))
                .remove_tag(s!("laptop-patch-1"), s!("a"), s!("food"))
                .add_tag(s!("laptop-patch-1"), s!("b"), s!("awesome-project")),
        ),
    ];

    let store = simple_store();

    for (device_id, meta) in expected_metas {
        assert_eq!(store.get_meta().unwrap(), meta);
    }
    for (patch_ref, patch) in expected_patches {
        assert_eq!(store.get_patch(patch_ref).unwrap(), patch);
    }
}

#[test]
fn check_repository_state() {
    let repository = Repository::from_store(simple_store());

    let current_timesheet = repository.get_current_timesheet();
    assert!(current_timesheet.is_ok());
    let current_timesheet = current_timesheet.unwrap();

    let mut expected_timesheet = Timesheet::new();
    expected_timesheet.insert_event(Event::new(dt!("2019-07-23T12:30:00Z"), sl!["lunch"]));
    expected_timesheet.insert_event(Event::new(
        dt!("2019-07-23T13:00:00Z"),
        sl!["work", "awesome-project"],
    ));

    assert_eq!(current_timesheet.flatten(), Ok(expected_timesheet));
}
