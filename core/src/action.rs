use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
pub type ActionRef = String;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TagRef {
    action: ActionRef,
    tag: String,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Action {
    #[serde(rename_all = "kebab-case")]
    CreateEvent {
        start: DateTime<Utc>,
        tags: Vec<String>,
    },

    #[serde(rename_all = "kebab-case")]
    DeleteEvent { create_event_id: ActionRef },

    #[serde(rename_all = "kebab-case")]
    ModifyEvent {
        event_id: ActionRef,
        add_tags: Option<Vec<String>>,
        delete_tags: Option<Vec<TagRef>>,
        set_start: Option<DateTime<Utc>>,
    },
}

impl TagRef {
    pub fn from_strs(action_ref: &str, tag: &str) -> Self {
        Self {
            action: action_ref.to_string(),
            tag: tag.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::offset::TimeZone;
    use toml;

    macro_rules! svec {
        ( $( $x:expr ),* ) => {
            {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push($x.to_string());
                )*
                temp_vec
            }
        };
    }

    #[test]
    fn read_create_event_from_toml() {
        let expected = Action::CreateEvent {
            start: Utc.ymd(2019, 07, 23).and_hms(12, 00, 00),
            tags: svec!["food", "lunch"],
        };
        let toml_str = r#"
            type = "create-event"
            start = "2019-07-23T12:00:00+00:00"
            tags = ["food", "lunch"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_delete_event_from_toml() {
        let expected = Action::DeleteEvent {
            create_event_id: "101".to_string(),
        };
        let toml_str = r#"
            type = "delete-event"
            create-event-id = "101"
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_modify_event_add_tags_from_toml() {
        let expected = Action::ModifyEvent {
            event_id: "101".to_string(),
            add_tags: Some(svec!["hello", "world"]),
            delete_tags: None,
            set_start: None,
        };
        let toml_str = r#"
            type = "modify-event"
            event-id = "101"
            add-tags = ["hello", "world"]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_modify_event_delete_tags_from_toml() {
        let expected = Action::ModifyEvent {
            event_id: "101".to_string(),
            add_tags: None,
            delete_tags: Some(vec![TagRef::from_strs("101", "lunch")]),
            set_start: None,
        };
        let toml_str = r#"
            type = "modify-event"
            event-id = "101"
            delete-tags = [{action = "101", tag = "lunch"}]
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }

    #[test]
    fn read_modify_event_set_start_from_toml() {
        let expected = Action::ModifyEvent {
            event_id: "101".to_string(),
            add_tags: None,
            delete_tags: None,
            set_start: Some(Utc.ymd(2019, 07, 24).and_hms(14, 00, 00)),
        };
        let toml_str = r#"
            type = "modify-event"
            event-id = "101"
            set-start = "2019-07-24T14:00:00+00:00"
        "#;
        assert_eq!(toml::de::from_str(toml_str), Ok(expected));
    }
}
