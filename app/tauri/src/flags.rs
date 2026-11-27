//! What this build ships with: `FLAGS`, written by the build script (`build.rs`, `build/flags.rs`) from `flags.toml`
//! and `FRESHER_FLAGS`. It is fixed at build time; nothing reads the file at runtime.

include!(concat!(env!("OUT_DIR"), "/flags.rs"));
