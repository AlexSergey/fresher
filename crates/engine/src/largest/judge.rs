//! Whether a row of the Largest screen can be chosen, and whether its folder opens: one pure function over what the
//! engine read of the row.

use std::path::Path;

use fresher_core::{Completeness, NodeInfo, NodeKind, Offer, Protected, Withheld, is_app_bundle, is_package};

/// What the judgement reads of one row.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Judged<'a> {
    pub(crate) node: &'a NodeInfo,
    pub(crate) path: &'a Path,
    /// What the folders between the home and the row are.
    pub(crate) above: Above,
    /// The home's `Library`, as located.
    pub(crate) library: Option<&'a Path>,
    /// The home's Trash, as located.
    pub(crate) trash: Option<&'a Path>,
    /// The cleaner's protection's answer for `path`.
    pub(crate) protection: Result<(), Protected>,
    /// A node a module claimed is inside the row, its category ignored or not.
    pub(crate) holds_claimed: bool,
    /// What the ignore list holds of the row.
    pub(crate) ignored: Ignored,
}

/// What the ignore list holds of a row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ignored {
    /// Nothing at, above or inside it.
    No,
    /// A path inside it: the row is withheld, but a folder still opens to what is not ignored.
    Inside,
    /// The row's path or a folder above it.
    AtOrAbove,
}

/// What the folders strictly between the home and a row make of it, the gravest first.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Above {
    /// Nothing: plain folders, or none.
    #[default]
    Plain,
    /// A module claimed one of them, its category ignored or not.
    Claimed,
    /// One of them has a hidden name or is a package: what is inside is an app's or a tool's own.
    Hidden,
    /// One of them is an app bundle.
    App,
}

impl Above {
    /// The folders strictly between `home` and `path`, by their names (a claim is the caller's to know); `Plain` for
    /// a path outside the home.
    pub(crate) fn of(home: &Path, path: &Path) -> Self {
        let Ok(inside) = path.strip_prefix(home) else { return Self::Plain };
        let mut folders: Vec<String> =
            inside.components().map(|part| part.as_os_str().to_string_lossy().into_owned()).collect();
        folders.pop();
        if folders.iter().any(|name| is_app_bundle(name)) {
            Self::App
        } else if folders.iter().any(|name| name.starts_with('.') || is_package(name)) {
            Self::Hidden
        } else {
            Self::Plain
        }
    }
}

/// The offer for the row and whether it opens in Folders: the first that holds of the thirteen rows of the design's
/// table, in order. Only a folder that is not a package ever opens. What is inside a claimed folder is `explained`
/// and what is inside a package is `hidden`, as the folder above it would be.
pub(crate) fn judge(input: &Judged<'_>) -> (Offer, bool) {
    let node = input.node;
    let at_or_below = |place: Option<&Path>| place.is_some_and(|place| input.path.starts_with(place));
    let openable = node.kind == NodeKind::Folder && !is_package(&node.name);
    let withheld_opening = |reason: Withheld, opens: bool| (Offer::Withheld(reason), opens && openable);
    let withheld = |reason: Withheld| {
        withheld_opening(
            reason,
            matches!(reason, Withheld::StandardFolder | Withheld::HoldsExplained | Withheld::ReadInPart),
        )
    };
    if node.owner.is_some() || input.above == Above::Claimed {
        return withheld(Withheld::Explained);
    }
    if at_or_below(input.library) {
        return withheld(Withheld::Library);
    }
    if at_or_below(input.trash) {
        return withheld(Withheld::Trash);
    }
    if node.name.starts_with('.') || input.above == Above::Hidden {
        return withheld(Withheld::Hidden);
    }
    if is_app_bundle(&node.name) || input.above == Above::App {
        return withheld(Withheld::App);
    }
    if node.kind == NodeKind::Link {
        return withheld(Withheld::Link);
    }
    if node.not_scanned.is_some() {
        return withheld(Withheld::NotSeen);
    }
    match input.protection {
        Err(Protected::WholeFolder) => return withheld(Withheld::StandardFolder),
        Err(Protected::Never | Protected::Ancestor) => return withheld(Withheld::Protected),
        Err(Protected::Ignored) | Ok(()) => {}
    }
    if input.holds_claimed {
        return withheld(Withheld::HoldsExplained);
    }
    if node.sizes.completeness != Completeness::Complete {
        return withheld(Withheld::ReadInPart);
    }
    if input.ignored == Ignored::AtOrAbove || input.protection == Err(Protected::Ignored) {
        return withheld(Withheld::Ignored);
    }
    if input.ignored == Ignored::Inside {
        return withheld_opening(Withheld::Ignored, true);
    }
    (Offer::Offered, openable)
}

#[cfg(test)]
#[path = "judge_tests.rs"]
mod tests;
