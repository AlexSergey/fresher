# Crates

**Since:** 2026-11-15

Candidate dependencies, checked against [`../../restrictions/dependencies.md`](../../restrictions/dependencies.md) on
2026-11-15. Versions are the latest seen that day; the plan that adds a crate pins its exact version.

| Crate                               | Licence          | Use                                           | Note                                                                     |
| ----------------------------------- | ---------------- | --------------------------------------------- | ------------------------------------------------------------------------ |
| `rayon` 1.12, `crossbeam-deque` 0.8 | MIT / Apache-2.0 | parallel scan                                 | work-stealing walker on `crossbeam-deque`                                |
| `jwalk` 0.9                         | MIT              | parallel sorted walk                          | slower than a custom work-stealing walker                                |
| `walkdir` 2.5, `ignore` 0.4         | Unlicense OR MIT | simple walks                                  | taken as MIT; `ignore` honours `.gitignore` — wrong for disk totals      |
| `libc`                              | MIT / Apache-2.0 | `getattrlistbulk`, `setiopolicy_np`, `statfs` | the scanner's FFI module                                                 |
| `objc2`, `objc2-foundation`         | MIT              | `NSFileManager`, URL resource keys            | `objc2-app-kit` is Zlib / Apache-2.0 / MIT                               |
| ~~`rusqlite` 0.43~~                 | MIT              | —                                             | not taken: the app keeps plain versioned files instead (plan 002)        |
| `blake3` 1.8                        | CC0 / Apache-2.0 | full content hash                             |                                                                          |
| `twox-hash` 2                       | MIT              | xxh3 for prefix and suffix hashes             | instead of `xxhash-rust`                                                 |
| ~~`xxhash-rust`~~                   | **BSL-1.0**      | —                                             | **not on our licence list**                                              |
| `image_hasher` 3.1                  | MIT / Apache-2.0 | perceptual hashes (similar images)            | optional feature                                                         |
| `reflink-copy`                      | MIT / Apache-2.0 | replace duplicates with APFS clones           |                                                                          |
| `trash` 5.3                         | MIT              | —                                             | no result URL, no list or restore on macOS: call `NSFileManager` instead |
| `plist` 1.10                        | MIT              | reading `Info.plist` and container metadata   |                                                                          |
| `rustc-hash`, `file-id`             | MIT / Apache-2.0 | fast hash sets, file identity                 |                                                                          |
| `sysinfo` 0.40                      | MIT              | running processes (items in use)              |                                                                          |
| `ts-rs` 12.0.1                      | MIT              | TypeScript types from the Rust types          | taken by plan 002 (pinned `=12.0.1`)                                     |
| `tauri-plugin-single-instance`      | MIT / Apache-2.0 | one instance of the app                       | taken by plan 002 (pinned `=2.5.5`)                                      |

Every new crate still goes through `cargo deny check` and cargo-machete, and gets its row in the licence notices
(`npm run notices:generate`, plan [013](../../plans/013-release/readme.md)).
