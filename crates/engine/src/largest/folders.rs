//! The Folders list: the folders of the home nothing changed in for long, found by the state's query and kept by the
//! judgement's verdict, then paged with the totals of all of them.

use std::path::PathBuf;

use fresher_core::{
    ByteSize, FolderRow, FoldersPage, FoldersQuery, LeftFor, MAX_PAGE, NodeId, NodeInfo, NodeKind, Offer, Sizes,
    Timestamp, Withheld,
};

use crate::largest::judge::Ignored;
use crate::largest::rule::HomeItems;
use crate::largest::{Ground, from_home};
use crate::results::add;

/// One folder left alone: what the state knows of it, where it is, and its newest change.
pub(crate) type Listed = (NodeInfo, PathBuf, Timestamp);

/// What the list does with a folder the query answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    /// A row of the list.
    List,
    /// Not a row, but the folders inside it may be.
    Enter,
    /// Neither it nor anything inside it.
    Drop,
}

/// The verdict for a folder, the ignore list read first: ignored wins, even over a folder holding explained items.
fn verdict(ignored: Ignored, judged: (Offer, bool)) -> Verdict {
    match (ignored, judged) {
        (Ignored::AtOrAbove, _) => Verdict::Drop,
        (Ignored::Inside, _) => Verdict::Enter,
        (_, (Offer::Offered | Offer::Withheld(Withheld::HoldsExplained), _)) => Verdict::List,
        (_, (Offer::Withheld(_), true)) => Verdict::Enter,
        (_, (Offer::Withheld(_), false)) => Verdict::Drop,
    }
}

/// The folders to list below the home, largest allocated size first, ties in path order: each folder the state's
/// query answers that a cleanup did not remove and the verdict lists, and what lies in the ones the verdict enters.
/// The home's query takes the home's `rule`; one entered folder is its own root, so it takes a rule with no `Library`
/// or Trash name, which only the home's top has.
pub(crate) fn left_alone(
    ground: &Ground<'_>,
    home: NodeId,
    rule: &HomeItems,
    min: ByteSize,
    before: Timestamp,
) -> Vec<Listed> {
    let below = HomeItems::new(None, None);
    let mut listed = Vec::new();
    let mut roots = vec![home];
    while let Some(root) = roots.pop() {
        let rule = if root == home { rule } else { &below };
        for found in ground.state.left_alone(root, rule, min, before).unwrap_or_default() {
            let (Some(node), Some(path)) = (ground.state.node(found.folder), ground.state.path(found.folder)) else {
                continue;
            };
            if ground.is_gone(&path) {
                continue;
            }
            match verdict(ground.ignored(&path), ground.judge(&node, &path)) {
                Verdict::List => listed.push((node, path, found.newest)),
                Verdict::Enter => roots.push(found.folder),
                Verdict::Drop => {}
            }
        }
    }
    listed.sort_by(|a, b| b.0.sizes.allocated.cmp(&a.0.sizes.allocated).then_with(|| a.1.cmp(&b.1)));
    listed
}

/// The page `query` asks of `listed`: the totals over every folder listed, the rows from the offset.
pub(crate) fn folders_page(
    listed: &[Listed],
    query: FoldersQuery,
    ground: &Ground<'_>,
    home: Sizes,
    min_size: ByteSize,
    days: u32,
) -> FoldersPage {
    let skip = usize::try_from(query.offset).unwrap_or(usize::MAX);
    let take = usize::try_from(query.limit.min(MAX_PAGE)).unwrap_or(0);
    let projects: Vec<bool> = listed.iter().map(|(_, path, _)| is_project(ground, path)).collect();
    let rows =
        listed.iter().zip(&projects).skip(skip).take(take).map(|(listed, &project)| row(listed, project, ground));
    FoldersPage {
        rows: rows.collect(),
        offset: query.offset,
        total: u32::try_from(listed.len()).unwrap_or(u32::MAX),
        sizes: listed.iter().fold(Sizes::ZERO, |sum, (node, ..)| add(sum, node.sizes)),
        projects: u32::try_from(projects.iter().filter(|&&project| project).count()).unwrap_or(u32::MAX),
        home,
        min_size,
        days,
    }
}

/// The page of a state that holds no home: nothing listed.
pub(crate) fn empty_folders(offset: u32, min_size: ByteSize, days: u32) -> FoldersPage {
    FoldersPage {
        rows: Vec::new(),
        offset,
        total: 0,
        sizes: Sizes::ZERO,
        projects: 0,
        home: Sizes::ZERO,
        min_size,
        days,
    }
}

fn row((node, path, newest): &Listed, project: bool, ground: &Ground<'_>) -> FolderRow {
    FolderRow {
        place: from_home(ground.home, path.parent().unwrap_or(path)),
        newest: *newest,
        left_for: LeftFor::between(*newest, ground.state.taken()),
        project,
        node: node.clone(),
    }
}

/// It holds a `.git` folder.
fn is_project(ground: &Ground<'_>, path: &std::path::Path) -> bool {
    let git = ground.state.find(&path.join(".git")).and_then(|id| ground.state.node(id));
    git.is_some_and(|node| matches!(node.kind, NodeKind::Folder | NodeKind::Collapsed))
}

#[cfg(test)]
#[path = "folders_tests.rs"]
mod tests;
