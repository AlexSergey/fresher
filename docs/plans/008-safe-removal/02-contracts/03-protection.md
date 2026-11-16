# 008/02/03 — Protection

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** core

The OS-neutral half of check 3 (and 4) of the guard: pure and component-wise. Which paths are protected on macOS is
plan 03's (`fresher-macos::protection`); this step only decides what "protected" means for a path.

## Files

- `crates/core/src/cleanup/protection.rs` (new):

  ```rust
  pub struct Protection { never: Vec<PathBuf>, whole: Vec<PathBuf>, except: Vec<PathBuf> }
  pub enum Protected { Never, Ancestor, WholeFolder }
  impl Protection {
      pub fn new(never: Vec<PathBuf>, whole: Vec<PathBuf>, except: Vec<PathBuf>) -> Result<Self, ProtectionError>;
      pub fn with_ignored(&self, ignored: &[PathBuf]) -> Self;     // ignored paths are `never` for the guard
      pub fn check(&self, path: &Path) -> Result<(), Protected>;
  }
  ```

  - `never`: the path itself, anything inside it and every ancestor of it are refused. `except` carves a place out of
    a `never` (`/usr/local` out of `/usr`); the carved-out place itself counts as `whole`: what is inside it may go,
    the place as a whole never. An `except` that is not inside a `never` is refused (`ProtectionError::LooseExcept`).
  - `whole`: the path itself and every ancestor of it are refused; what is inside may go.
  - Comparison by `Path::components`, never by string prefix; every path must be absolute, with no `.` or `..`
    (`ProtectionError::NotPlain`); a volume that ignores case is handled by the caller giving canonical paths (the
    guard checks the physical path, plan 04).
- `crates/core/src/lib.rs` — the exports.

## Produces

The check plan 04's guard calls with the macOS list of plan 03 and the ignore list of plan 005.

## Checks

- Negative (first): `a_protected_path_is_refused`, `a_path_inside_a_protected_one_is_refused`,
  `an_ancestor_of_a_protected_path_is_refused` (`/`, `/Users`, the home above `~/Library/Mobile Documents`),
  `a_whole_folder_is_refused_as_a_whole`, `an_ancestor_of_a_whole_folder_is_refused`,
  `a_carved_out_place_itself_is_refused` (`/usr/local`), `a_relative_or_dotted_path_is_not_accepted`,
  `a_shared_prefix_is_not_inside` (`/Users/ann` vs `/Users/anna`), `an_ignored_path_and_its_ancestors_are_refused`.
- Positive: `an_item_inside_a_whole_folder_may_go` (`~/Library/Caches/x`), `an_item_inside_a_carve_out_may_go`
  (`/usr/local/x`), `an_unrelated_path_passes`.
- Property: `check_agrees_with_a_naive_component_model` (random component paths against a reference written as plain
  loops); `an_ancestor_of_anything_never_passes`.

## Conclusions

- Built in `crates/core/src/cleanup/protection.rs` with the step's checks, one more case
  (`a_never_place_inside_a_carve_out_is_still_refused`), and the two property tests (the model is written as plain
  loops over lists of names).
- Changed from the step, to fail closed and to keep the guard's skip reasons apart:
  - **The ignored paths are kept apart from `never`** and answered with their own **`Protected::Ignored`**, so check 4
    reports `Ignored` and not `Protected`; they are refused the same way (the path, what is inside, its ancestors).
  - **`with_ignored` returns a `Result`**: an ignored path that is not absolute and plain is refused
    (`ProtectionError::NotPlain`) instead of being kept as a rule that matches nothing, which would remove what the
    user asked to keep.
- Not spelled out by the step, taken the simplest or the safest way:
  - **The order of the answers**: a `never` place or inside one (`Never`), an ancestor of one (`Ancestor`), a `whole`
    or carved-out place or an ancestor of one (`WholeFolder`), then the ignored paths (`Ignored`). The home folder,
    which is a `whole` folder and an ancestor of `never` places, answers `Ancestor`. The cleaner (04/02) maps `Never`
    and `Ancestor` to `SkipReason::Protected`, `WholeFolder` to `WholeFolder`, `Ignored` to `Ignored`.
  - **A path that is not absolute and plain is refused as `Never`** by `check` (fail closed). "Plain" is checked on
    the raw names too, since `Path::components` drops a `.` inside a path.
  - An `except` equal to its `never` place is `LooseExcept` too: a carve-out must be strictly inside.
  - Containment is `Path::starts_with`, which compares components.

