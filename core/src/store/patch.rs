use crate::Tag;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PatchRef = Uuid;
type EventRef = String;
type Set<T> = std::collections::HashSet<T>;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Patch {
    pub id: Uuid,

    #[serde(default)]
    pub add_start: Set<AddStart>,

    #[serde(default)]
    pub remove_start: Set<RemoveStart>,

    #[serde(default)]
    pub add_tag: Set<AddTag>,

    #[serde(default)]
    pub remove_tag: Set<RemoveTag>,

    #[serde(default)]
    pub create_event: Set<CreateEvent>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct AddStart {
    pub parent: PatchRef,
    pub event: EventRef,
    pub time: DateTime<Utc>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct RemoveStart {
    pub patch: PatchRef,
    pub event: EventRef,
    pub time: DateTime<Utc>,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct AddTag {
    pub parent: PatchRef,
    pub event: EventRef,
    pub tag: Tag,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct RemoveTag {
    pub patch: PatchRef,
    pub event: EventRef,
    pub tag: Tag,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct CreateEvent {
    pub event: EventRef,
    pub start: DateTime<Utc>,
    pub tags: Vec<Tag>,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            add_start: Set::new(),
            remove_start: Set::new(),
            add_tag: Set::new(),
            remove_tag: Set::new(),
            create_event: Set::new(),
        }
    }

    pub fn patch_ref(&self) -> &PatchRef {
        &self.id
    }

    pub fn parents(&self) -> Set<PatchRef> {
        self.add_start
            .iter()
            .map(|x| &x.parent)
            .chain(self.remove_start.iter().map(|x| &x.patch))
            .chain(self.add_tag.iter().map(|x| &x.parent))
            .chain(self.remove_tag.iter().map(|x| &x.patch))
            .cloned()
            .collect()
    }

    pub fn add_start(mut self, parent: PatchRef, event: EventRef, time: DateTime<Utc>) -> Self {
        self.add_start.insert(AddStart {
            parent,
            event,
            time,
        });
        self
    }

    pub fn remove_start(mut self, patch: PatchRef, event: EventRef, time: DateTime<Utc>) -> Self {
        self.remove_start.insert(RemoveStart { patch, event, time });
        self
    }

    pub fn add_tag(mut self, parent: PatchRef, event: EventRef, tag: String) -> Self {
        self.add_tag.insert(AddTag { parent, event, tag });
        self
    }

    pub fn remove_tag(mut self, patch: PatchRef, event: EventRef, tag: String) -> Self {
        self.remove_tag.insert(RemoveTag { patch, event, tag });
        self
    }

    pub fn create_event(
        mut self,
        event: EventRef,
        start: DateTime<Utc>,
        tags: Vec<String>,
    ) -> Self {
        self.create_event.insert(CreateEvent { event, start, tags });
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::{TimeZone, Utc};

    macro_rules! s (
        { $stuff:expr } => {
            {
                $stuff.to_string()
            }
         };
    );

    #[test]
    fn read_patch_with_create_event_toml() {
        let expected = Patch::new().create_event(
            s!("a"),
            Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
            vec![s!("work"), s!("coding")],
        );

        let toml_str = r#"
            [[create-event]]
            event = "a"
            start = "2019-07-24T14:00:00+00:00"
            tags = ["work", "coding"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_patch_with_all_fields_toml() {
        let expected = Patch::new()
            .add_start(s!("0"), s!("a"), Utc.ymd(2019, 7, 24).and_hms(14, 0, 0))
            .remove_start(s!("0"), s!("a"), Utc.ymd(2019, 7, 24).and_hms(14, 0, 0))
            .add_tag(s!("0"), s!("a"), s!("work"))
            .remove_tag(s!("0"), s!("a"), s!("coding"))
            .create_event(
                s!("a"),
                Utc.ymd(2019, 7, 24).and_hms(14, 0, 0),
                vec![s!("work"), s!("coding")],
            );

        let toml_str = r#"
            [[add-start]]
            parent = "0"
            event = "a"
            time = "2019-07-24T14:00:00+00:00"

            [[remove-start]]
            patch = "0"
            event = "a"
            time = "2019-07-24T14:00:00+00:00"

            [[add-tag]]
            parent = "0"
            event = "a"
            tag = "work"

            [[remove-tag]]
            patch = "0"
            event = "a"
            tag = "coding"

            [[create-event]]
            event = "a"
            start = "2019-07-24T14:00:00+00:00"
            tags = ["work", "coding"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
