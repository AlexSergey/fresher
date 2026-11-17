//! The map by place: the folder tree from a root, each row with how much of it is explained.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{NodeInfo, Sizes};

/// One page of the by-place view of a folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PlaceView {
    /// The node opened and the folders the automatic descent went through, the outermost first.
    pub path: Vec<NodeInfo>,
    pub rows: Vec<PlaceRow>,
    pub offset: u32,
    /// How many rows the opened folder has in all.
    pub total: u32,
}

/// One child of the opened folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct PlaceRow {
    pub node: NodeInfo,
    /// How much of the node the modules explain.
    pub explained: Sizes,
}

/// How a place opens: down through the folders that are almost all one child, or at the folder asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Descent {
    /// Down through every folder that is almost all one child: a folder row opened.
    Through,
    /// At the folder asked: a step of the breadcrumb, a folder the user already went through.
    Stay,
}
