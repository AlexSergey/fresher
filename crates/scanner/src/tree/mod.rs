//! The tree of one scan, in an arena: folders and kept files as nodes, small files folded into their folder.

pub(crate) mod access;
mod apply;
pub(crate) mod arena;
pub(crate) mod fold;
pub(crate) mod links;
pub(crate) mod names;
pub(crate) mod node;
mod opaque;
#[cfg(test)]
pub(crate) mod testing;

/// Why the tree refused a change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum TreeError {
    /// A name longer than any file system allows.
    #[error("a name of {len} bytes is too long")]
    NameTooLong { len: usize },
    /// The arena has no index or name offset left.
    #[error("the tree is full")]
    Full,
    #[error("no such node")]
    NoSuchNode,
    #[error("the node is not a folder")]
    NotAFolder,
    /// A folder that already has children cannot be made opaque.
    #[error("the folder already has children")]
    NotOpaque,
    /// The tree was frozen; it is not built any further.
    #[error("the tree is frozen")]
    Frozen,
}
