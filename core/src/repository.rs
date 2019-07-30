use chrono::{DateTime, Utc};
use multihash::Multihash;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::PathBuf,
};
use toml;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
struct Reference(String);

type PatchRef = Reference;
type ActionRef = Reference;
type TagRef = (ActionRef, String);

trait Repository {
    type Store: RepositoryStore;

    fn actions_in_order(&self) -> Vec<(ActionRef, Action)>;
}

struct SimpleRepository<S: Store> {
    device_id: String,
    store: S,
}


impl<S: RepositoryStore> SimpleRepository<S> {
    fn new(store: S, device_id: String) -> Self {
        Self { device_id, store }
    }
}

impl<S: RepositoryStore> Repository for SimpleRepository<S> {
    type Store = S;

    fn actions_in_order(&self) -> Vec<(ActionRef, Action)> {
        let mut actions = vec![];
        actions
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::TimeZone;

    #[derive(Default)]
    struct InMemoryStore {
        meta: HashMap<String, Meta>,
        patches: HashMap<String, Patch>,
    }

    impl InMemoryStore {}

    impl RepositoryStore for InMemoryStore {
        fn get_device_meta(&self, device_id: &str) -> Result<Meta, ()> {
            match self.meta.get(device_id) {
                Some(s) => Ok(s.clone()),
                None => Err(()),
            }
        }
        fn get_patch(&self, hash: &str) -> Result<Patch, ()> {
            match self.patches.get(hash) {
                Some(s) => Ok(s.clone()),
                None => Err(()),
            }
        }
    }

    #[test]
    fn one_action_repo() {
        let mut store = InMemoryStore::default();
        store.meta.insert(
            "test".to_string(),
            Meta {
                patches: vec![Reference("a".to_string())],
            },
        );

        let create_event = Action::CreateEvent {
            start: Utc.ymd(2019, 7, 23).and_hms(0, 0, 0),
            tags: ["augr", "coding"].iter().map(|s| s.to_string()).collect(),
        };
        let create_event_ref = Reference("a".to_string());
        let mut actions = HashMap::new();
        actions.insert(create_event_ref.clone(), create_event.clone());

        store.patches.insert(
            "a".to_string(),
            Patch {
                parents: vec![],
                actions,
            },
        );

        let repo = SimpleRepository::new(store, "test".to_string());
        assert_eq!(
            repo.actions_in_order(),
            vec![(create_event_ref, create_event)]
        );
    }
}
