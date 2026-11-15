//! What a finding is about.

use std::path::PathBuf;

use serde::Serialize;
use ts_rs::TS;

use crate::ToolId;

/// The thing a finding is about: a file or folder, or an object of an external tool (a Docker image).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum Subject {
    Path(PathBuf),
    External { tool: ToolId, id: String },
}

#[cfg(test)]
mod tests {
    mod positive {
        use serde_json::json;

        use crate::{Subject, ToolId};

        #[test]
        fn serializes_with_a_kind_tag() {
            let image = Subject::External { tool: ToolId::new("docker").unwrap(), id: "sha256:1".into() };

            assert_eq!(
                serde_json::to_value(Subject::Path("/a".into())).unwrap(),
                json!({ "kind": "path", "value": "/a" })
            );
            assert_eq!(
                serde_json::to_value(image).unwrap(),
                json!({ "kind": "external", "value": { "tool": "docker", "id": "sha256:1" } })
            );
        }
    }
}
