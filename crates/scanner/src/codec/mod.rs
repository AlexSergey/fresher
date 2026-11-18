//! The cache body: a frozen tree written in depth-first order with variable-length integers and front-coded names,
//! and read back into a tree that freezes to the same totals. The store wraps the body in its envelope.

pub(crate) mod decode;
pub(crate) mod encode;
pub(crate) mod fit;
pub(crate) mod header;
mod stamp;
mod varint;

#[cfg(test)]
pub(crate) mod testing;
#[cfg(test)]
mod tests;

use crate::tree::TreeError;

/// Why a body could not be written or read. A body that cannot be read costs one full scan, never a wrong tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum CodecError {
    #[error("the body ends early")]
    Truncated,
    #[error("a number is written with more bytes than it needs")]
    OverlongNumber,
    #[error("a number is out of its range")]
    OutOfRange,
    #[error("a count is larger than the rest of the body")]
    CountTooLarge,
    #[error("the body has tree format {found}, this build reads {}", header::TREE_FORMAT)]
    OtherFormat { found: u16 },
    #[error("a name is not valid or not in order")]
    BadName,
    #[error("a node's flags are not valid")]
    BadFlags,
    #[error("a node names a claim the table does not hold")]
    UnknownOwner,
    #[error("bytes follow the end of the body")]
    TrailingBytes,
    #[error("the body of {body} bytes, {packed} compressed, does not fit the cap even with every subtree collapsed")]
    TooLarge { body: u64, packed: u64 },
    #[error("only a frozen tree is written")]
    NotFrozen,
    #[error("the tree refused the body: {0}")]
    Tree(#[from] TreeError),
}
