//! The app's own data folder of Fresher: settings, the scan cache, the cleanup journal and other versioned files.
//!
//! The only crate that touches the data folder. Every file is written atomically (the journal is appended to) and carries its `format` and
//! `version`. The app passes the folder in; the store never looks it up itself and deletes nothing.

#![forbid(unsafe_code)]

mod atomic;
mod data_folder;
mod envelope;
mod footprint;
mod journal;
mod migrate;
mod own_files;
mod scan_cache;
mod settings_file;
mod versioned;

pub use data_folder::{DataFolder, DataFolderError};
pub use footprint::{FOOTPRINT_BUDGET, Footprint};
pub use journal::FileCleanupJournal;
pub use own_files::atomically_written_files;
pub use scan_cache::{CachePolicy, FileScanCache};
pub use settings_file::FileSettingsStore;
