use crate::{Action, ActionRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PatchRef = String;

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub struct Patch {
    #[serde(default)]
    parents: Vec<PatchRef>,
    actions: HashMap<ActionRef, Action>,
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::{TimeZone, Utc};

    macro_rules! map(
        { $($key:expr => $value:expr),+ } => {
            {
                let mut m = ::std::collections::HashMap::new();
                $(
                    m.insert($key.into(), $value);
                )+
                m
            }
         };
    );

    #[test]
    fn read_standalone_single_action_patch_from_toml() {
        let expected = Patch {
            parents: vec![],
            actions: map! {
                "102" => Action::ModifyEvent {
                    event_id: "101".to_string(),
                    add_tags: None,
                    delete_tags: None,
                    set_start: Some(Utc.ymd(2019, 07, 24).and_hms(14, 00, 00)),
                }
            },
        };
        let toml_str = r#"
            [actions.102]
            type = "modify-event"
            event-id = "101"
            set-start = "2019-07-24T14:00:00+00:00"
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_standalone_multi_action_patch_from_toml() {
        let expected = Patch {
            parents: vec![],
            actions: map! {
                "102" => Action::ModifyEvent {
                    event_id: "101".to_string(),
                    add_tags: None,
                    delete_tags: None,
                    set_start: Some(Utc.ymd(2019, 07, 24).and_hms(14, 00, 00)),
                },
                "103" => Action::CreateEvent {
                    start: Utc.ymd(2019, 07, 24).and_hms(15, 00, 00),
                    tags: vec!["travel".to_string()],
                }
            },
        };
        let toml_str = r#"
            [actions.102]
            type = "modify-event"
            event-id = "101"
            set-start = "2019-07-24T14:00:00+00:00"

            [actions.103]
            type = "create-event"
            start = "2019-07-24T15:00:00+00:00"
            tags = ["travel"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_patch_with_ancestors_from_toml() {
        let expected = Patch {
            parents: vec!["laptop-1".to_string()],
            actions: map! {
                "102" => Action::ModifyEvent {
                    event_id: "101".to_string(),
                    add_tags: None,
                    delete_tags: None,
                    set_start: Some(Utc.ymd(2019, 07, 24).and_hms(14, 00, 00)),
                }
            },
        };
        let toml_str = r#"
            parents = [ "laptop-1" ]

            [actions.102]
            type = "modify-event"
            event-id = "101"
            set-start = "2019-07-24T14:00:00+00:00"
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

}
