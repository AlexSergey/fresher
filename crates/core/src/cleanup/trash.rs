//! The Trash as the scan screen shows it: what waits in each Trash, and how much of it the app's cleanups put there.
//! Information only: nothing here empties a Trash.

use serde::Serialize;
use ts_rs::TS;

use crate::{Sizes, Timestamp};

/// The answer of `get_trash`: each Trash that holds something, the home volume's first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct TrashView {
    pub trashes: Vec<TrashLine>,
}

/// One Trash: what waits there, and the part of it the app's cleanups moved there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrashLine {
    /// What `open_trash` names it by: its place in this answer.
    pub index: u32,
    /// Its volume's name; `None` when the volume could not be read.
    pub volume: Option<String>,
    /// Everything in this Trash: the last scan's measure plus what cleanups moved there since and is still there;
    /// `None` when no scan measured it.
    pub waiting: Option<Sizes>,
    /// What the app's cleanups moved there and is still there, found again now.
    pub from_cleanups: Sizes,
    /// With Finder's 30-day setting on: when the last of `from_cleanups` leaves the Trash by itself; `None` otherwise.
    pub gone_by: Option<Timestamp>,
}

#[cfg(test)]
mod tests {
    use crate::{ByteSize, Sizes, Timestamp, TrashLine};

    fn line() -> TrashLine {
        TrashLine {
            index: 0,
            volume: Some("Macintosh HD".into()),
            waiting: Some(Sizes::complete(ByteSize::new(9_000), ByteSize::new(12_288))),
            from_cleanups: Sizes::complete(ByteSize::new(4_000), ByteSize::new(4_096)),
            gone_by: Some(Timestamp::from_secs(2_592_000)),
        }
    }

    mod negative {
        use super::line;
        use crate::TrashView;

        #[test]
        fn a_serialized_view_holds_no_path_and_no_snake_case_key() {
            let view = TrashView { trashes: vec![line()] };

            let json = serde_json::to_string(&view).unwrap();

            assert!(!json.contains('/'), "{json}");
            assert!(!json.contains("from_cleanups"), "{json}");
            assert!(!json.contains("gone_by"), "{json}");
        }
    }

    mod positive {
        use serde_json::{Value, json};

        use super::line;
        use crate::{TrashLine, TrashView};

        #[test]
        fn a_line_serializes_from_cleanups_and_gone_by() {
            let json = serde_json::to_value(line()).unwrap();

            assert_eq!(json["index"], json!(0));
            assert_eq!(json["volume"], json!("Macintosh HD"));
            assert_eq!(json["fromCleanups"]["allocated"], json!(4_096));
            assert_eq!(json["goneBy"], json!(2_592_000));
        }

        #[test]
        fn a_line_with_nothing_measured_serializes_waiting_as_null() {
            let unmeasured = TrashLine { waiting: None, gone_by: None, ..line() };

            let json = serde_json::to_string(&TrashView { trashes: vec![unmeasured] }).unwrap();

            assert!(json.contains(r#""waiting":null"#), "{json}");
            assert!(json.contains(r#""goneBy":null"#), "{json}");
            assert!(serde_json::from_str::<Value>(&json).unwrap()["trashes"].is_array());
        }
    }
}
