//! The read-only tree of the last committed scan: what the storage screens and the search read.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CategoryId, Sizes, TextKey, Timestamp, Unreadable};

/// The tree of one committed scan. Never changes: a new scan commits a new state with a new id.
pub trait DiskState: Send + Sync {
    /// The id every `NodeId` of this state carries.
    fn id(&self) -> StateId;
    /// When the scan that made this state ended.
    fn taken(&self) -> Timestamp;
    /// The scan roots, in the order scanned.
    fn roots(&self) -> Vec<NodeId>;
    /// The node `id`; `None` for an id of another state or no node.
    fn node(&self, id: NodeId) -> Option<NodeInfo>;
    /// One page of the children of `id`, in `order`; `None` for an id of another state or no node.
    fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage>;
    /// The kept file records, the largest first.
    fn largest_files(&self, limit: u32) -> Vec<NodeInfo>;
    /// The nodes whose name contains `query`, case ignored, the largest first.
    fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo>;
    /// The top-most folders that were not scanned, with why and how many folders each hides.
    fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace>;
    /// The full path of `id`; `None` for an id of another state or no node.
    fn path(&self, id: NodeId) -> Option<PathBuf>;
    /// Every node a category claimed, with its owner; an ancestor always before its descendants.
    fn owned(&self) -> Vec<(NodeId, CategoryId)>;
    /// The node at `path`, compared by components; `None` outside every root or for no node.
    fn find(&self, path: &Path) -> Option<NodeId>;
}

/// The id of one committed state in a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct StateId(u32);

impl StateId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A node of one state; another state refuses it, like a `FindingId` of another session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
pub struct NodeId {
    state: StateId,
    index: u32,
}

impl NodeId {
    pub const fn new(state: StateId, index: u32) -> Self {
        Self { state, index }
    }

    pub const fn state(self) -> StateId {
        self.state
    }

    pub const fn index(self) -> u32 {
        self.index
    }
}

/// One node as the UI sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
    pub id: NodeId,
    /// The name for display: lossy UTF-8 (the scanner keeps the bytes).
    pub name: String,
    pub kind: NodeKind,
    /// The node's own size; a folder's holds everything inside it.
    pub sizes: Sizes,
    /// The files inside a folder, at every depth; 0 for anything else.
    #[ts(type = "number")]
    pub files: u64,
    /// The folders inside a folder, at every depth; 0 for anything else.
    #[ts(type = "number")]
    pub folders: u64,
    /// The category that claimed the node in the scan.
    pub owner: Option<CategoryId>,
    pub modified: Option<Timestamp>,
    /// Why the node was not scanned; `None` when it was.
    pub not_scanned: Option<NotScanned>,
}

/// What a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum NodeKind {
    Folder,
    /// A file large enough to be kept as its own record.
    File,
    /// A symbolic link: listed, never followed.
    Link,
    /// A subtree folded into one node to keep the cache within its cap.
    Collapsed,
}

/// The order of a page of children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ChildOrder {
    /// The largest first (allocated size), then by name.
    BySize,
    /// By name.
    ByName,
}

/// Why a folder was not scanned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum NotScanned {
    /// It could not be read.
    Unreadable(Unreadable),
    /// The walk did not open it: opening it would raise a system prompt.
    Avoided,
    /// It is on another volume than its scan root.
    OtherVolume,
    /// A folder of another user, in a place outside the scan roots.
    OthersFolder,
    /// The system's own items (the user's temporary folder's `TemporaryItems`): private to macOS, never opened.
    SystemItems,
}

impl NotScanned {
    /// Every reason, in the order of the enum.
    pub const ALL: [Self; 7] = [
        Self::Unreadable(Unreadable::NoPermission),
        Self::Unreadable(Unreadable::Guarded),
        Self::Unreadable(Unreadable::Failed),
        Self::Avoided,
        Self::OtherVolume,
        Self::OthersFolder,
        Self::SystemItems,
    ];

    /// Why, in plain words, as the place view says it under the folder:
    /// `core.storage.not-scanned.<kind>`, an unreadable folder's with its reason (`unreadable.guarded`).
    pub fn text_key(self) -> TextKey {
        let segment = match self {
            Self::Unreadable(Unreadable::NoPermission) => "unreadable.no-permission",
            Self::Unreadable(Unreadable::Guarded) => "unreadable.guarded",
            Self::Unreadable(Unreadable::Failed) => "unreadable.failed",
            Self::Avoided => "avoided",
            Self::OtherVolume => "other-volume",
            Self::OthersFolder => "others-folder",
            Self::SystemItems => "system-items",
        };
        TextKey::from_valid(format!("core.storage.not-scanned.{segment}"))
    }
}

/// One page of the children of a node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct NodePage {
    pub nodes: Vec<NodeInfo>,
    pub offset: u32,
    /// How many children the node has in all.
    pub total: u32,
}

/// A top-most folder that was not scanned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UnreadablePlace {
    pub node: NodeInfo,
    pub reason: NotScanned,
    /// The folders under it that were not scanned with it.
    #[ts(type = "number")]
    pub hidden_folders: u64,
}

#[cfg(test)]
mod tests {
    mod positive {
        use serde_json::json;

        use crate::{
            ByteSize, NodeId, NodeInfo, NodeKind, NotScanned, Sizes, StateId, Timestamp, Unreadable, UnreadablePlace,
        };

        fn node() -> NodeInfo {
            NodeInfo {
                id: NodeId::new(StateId::new(2), 7),
                name: "Caches".into(),
                kind: NodeKind::Folder,
                sizes: Sizes::complete(ByteSize::new(10), ByteSize::new(4096)),
                files: 3,
                folders: 1,
                owner: None,
                modified: Some(Timestamp::from_secs(100)),
                not_scanned: Some(NotScanned::Unreadable(Unreadable::NoPermission)),
            }
        }

        #[test]
        fn node_info_serializes_in_camel_case() {
            assert_eq!(
                serde_json::to_value(node()).unwrap(),
                json!({
                    "id": { "state": 2, "index": 7 },
                    "name": "Caches",
                    "kind": "folder",
                    "sizes": { "logical": 10, "allocated": 4096, "reclaimable": 4096, "completeness": "complete" },
                    "files": 3,
                    "folders": 1,
                    "owner": null,
                    "modified": 100,
                    "notScanned": { "kind": "unreadable", "value": "no-permission" },
                })
            );
        }

        #[test]
        fn an_unreadable_place_serializes_its_reason_and_hidden_folders() {
            let place = UnreadablePlace { node: node(), reason: NotScanned::Avoided, hidden_folders: 4 };

            let json = serde_json::to_value(place).unwrap();

            assert_eq!(json["reason"], json!({ "kind": "avoided" }));
            assert_eq!(json["hiddenFolders"], 4);
        }

        #[test]
        fn each_reason_not_scanned_has_its_own_text() {
            let keys: std::collections::BTreeSet<_> = NotScanned::ALL.map(NotScanned::text_key).into_iter().collect();

            assert_eq!(keys.len(), NotScanned::ALL.len());
            assert_eq!(
                NotScanned::Unreadable(Unreadable::Guarded).text_key().as_str(),
                "core.storage.not-scanned.unreadable.guarded"
            );
        }

        #[test]
        fn a_node_id_knows_its_state() {
            let id = NodeId::new(StateId::new(3), 9);

            assert_eq!((id.state().get(), id.index()), (3, 9));
        }
    }
}
