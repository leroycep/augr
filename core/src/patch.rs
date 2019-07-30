use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type PatchRef = String;
type EventRef = String;
pub type Tag = String;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct Patch {
    #[serde(default)]
    add_start: Vec<AddStart>,

    #[serde(default)]
    remove_start: Vec<RemoveStart>,

    #[serde(default)]
    add_tag: Vec<AddTag>,

    #[serde(default)]
    remove_tag: Vec<RemoveTag>,

    #[serde(default)]
    create_event: Vec<CreateEvent>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct AddStart {
    parent: PatchRef,
    event: EventRef,
    time: DateTime<Utc>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct RemoveStart {
    patch: PatchRef,
    event: EventRef,
    time: DateTime<Utc>,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct AddTag {
    parent: PatchRef,
    event: EventRef,
    tag: Tag,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct RemoveTag {
    patch: PatchRef,
    event: EventRef,
    tag: Tag,
}

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct CreateEvent {
    event: EventRef,
    start: DateTime<Utc>,
    tags: Vec<Tag>,
}

impl Patch {
    pub fn new() -> Self {
        Self {
            add_start: Vec::new(),
            remove_start: Vec::new(),
            add_tag: Vec::new(),
            remove_tag: Vec::new(),
            create_event: Vec::new(),
        }
    }

    pub fn add_start(&mut self, parent: PatchRef, event: EventRef, time: DateTime<Utc>) {
        self.add_start.push(AddStart {
            parent,
            event,
            time,
        })
    }

    pub fn remove_start(&mut self, patch: PatchRef, event: EventRef, time: DateTime<Utc>) {
        self.remove_start.push(RemoveStart { patch, event, time })
    }

    pub fn add_tag(&mut self, parent: PatchRef, event: EventRef, tag: String) {
        self.add_tag.push(AddTag { parent, event, tag })
    }

    pub fn remove_tag(&mut self, patch: PatchRef, event: EventRef, tag: String) {
        self.remove_tag.push(RemoveTag { patch, event, tag })
    }

    pub fn create_event(&mut self, event: EventRef, start: DateTime<Utc>, tags: Vec<String>) {
        self.create_event.push(CreateEvent { event, start, tags })
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
        let mut expected = Patch::new();
        expected.create_event(
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
        let mut expected = Patch::new();
        expected.add_start(s!("0"), s!("a"), Utc.ymd(2019, 7, 24).and_hms(14, 0, 0));
        expected.remove_start(s!("0"), s!("a"), Utc.ymd(2019, 7, 24).and_hms(14, 0, 0));
        expected.add_tag(s!("0"), s!("a"), s!("work"));
        expected.remove_tag(s!("0"), s!("a"), s!("coding"));
        expected.create_event(
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
