//! The Largest screen's types: what a row is called, what the screen asks and is answered, and the core's own category
//! for the rows the user picks. No IO: the engine fills these in.

mod folders;
mod kind;
mod own_file;
mod page;

pub use folders::{FolderRow, FoldersPage, FoldersQuery, LeftFor};
pub use kind::{ItemKind, archive_stem, is_app_bundle, is_package, kind_of};
pub use own_file::{OWN_FILE_ID, own_file, own_file_reason, own_file_title};
pub use page::{
    KindCount, LargestPage, LargestQuery, LargestRow, MAX_ITEMS, MAX_PAGE, Offer, OwnPlace, OwnRow, PlaceCount,
    Unchanged, Withheld,
};
