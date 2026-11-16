//! The app's own data folder of Fresher: settings, the scan cache and other versioned files.
//!
//! The only crate that touches the data folder. Every file is written atomically and carries its `format` and
//! `version`. The app passes the folder in; the store never looks it up itself and deletes nothing.

#![forbid(unsafe_code)]

mod atomic;
mod data_folder;
mod envelope;
mod footprint;
mod migrate;
mod scan_cache;
mod settings_file;
mod versioned;

pub use data_folder::{DataFolder, DataFolderError};
pub use footprint::{FOOTPRINT_BUDGET, Footprint};
pub use scan_cache::{CachePolicy, FileScanCache};
pub use settings_file::FileSettingsStore;
