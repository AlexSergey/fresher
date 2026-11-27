//! A page of the Files list from the items of a state: filtered, counted and summed in memory.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use fresher_core::{
    ByteSize, ItemKind, KindCount, LargestPage, LargestQuery, LargestRow, MAX_PAGE, NodeId, NodeKind, PlaceCount,
    Sizes, archive_stem,
};

use crate::largest::items::{Item, Items};
use crate::largest::{Ground, from_home};
use crate::results::add;

/// The page `query` asks of `items`, leaving out the ignored paths and what a cleanup removed of the picks.
pub(crate) fn files_page(items: &Items, query: &LargestQuery, ground: &Ground<'_>) -> LargestPage {
    let cutoff = query.unchanged.before(ground.state.taken());
    let kept: Vec<&Item> =
        items.list.iter().filter(|item| !ground.is_ignored(&item.path) && !ground.is_gone(&item.path)).collect();
    let of_kind = |item: &Item| query.kind.is_none_or(|kind| item.kind == kind);
    let in_place = |item: &Item| query.place.is_none_or(|place| item.top == Some(place));
    let unchanged = |item: &Item| cutoff.is_none_or(|cutoff| item.modified.is_some_and(|modified| modified < cutoff));
    let mut kinds: BTreeMap<ItemKind, u32> = BTreeMap::new();
    let mut places: HashMap<NodeId, u32> = HashMap::new();
    let mut rows: Vec<&Item> = Vec::new();
    for &item in &kept {
        if in_place(item) && unchanged(item) {
            count(kinds.entry(item.kind).or_default());
        }
        if let Some(top) = item.top.filter(|_| of_kind(item) && unchanged(item)) {
            count(places.entry(top).or_default());
        }
        if of_kind(item) && in_place(item) && unchanged(item) {
            rows.push(item);
        }
    }
    let sizes =
        rows.iter().filter_map(|item| ground.state.node(item.id)).fold(Sizes::ZERO, |sum, node| add(sum, node.sizes));
    let skip = usize::try_from(query.offset).unwrap_or(usize::MAX);
    let take = usize::try_from(query.limit.min(MAX_PAGE)).unwrap_or(0);
    LargestPage {
        rows: rows.iter().skip(skip).take(take).filter_map(|item| row(item, ground)).collect(),
        offset: query.offset,
        total: u32::try_from(rows.len()).unwrap_or(u32::MAX),
        sizes,
        kinds: kinds.into_iter().map(|(kind, count)| KindCount { kind, count }).collect(),
        places: place_counts(places, ground),
        truncated: items.truncated,
        min_size: items.min,
    }
}

/// The page of a state that holds no home: nothing listed.
pub(crate) fn empty_page(offset: u32, min_size: ByteSize) -> LargestPage {
    LargestPage {
        rows: Vec::new(),
        offset,
        total: 0,
        sizes: Sizes::ZERO,
        kinds: Vec::new(),
        places: Vec::new(),
        truncated: false,
        min_size,
    }
}

fn count(counter: &mut u32) {
    *counter = counter.saturating_add(1);
}

/// The folders with their counts, the most items first, then by name.
fn place_counts(places: HashMap<NodeId, u32>, ground: &Ground<'_>) -> Vec<PlaceCount> {
    let mut counts: Vec<PlaceCount> = places
        .into_iter()
        .filter_map(|(folder, count)| Some(PlaceCount { folder: ground.state.node(folder)?, count }))
        .collect();
    counts.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.folder.name.cmp(&b.folder.name)));
    counts
}

fn row(item: &Item, ground: &Ground<'_>) -> Option<LargestRow> {
    let node = ground.state.node(item.id)?;
    let (offer, _) = ground.judge_reached(&node, &item.path);
    let parent = item.path.parent().unwrap_or(&item.path);
    let unpacked = (item.kind == ItemKind::Archive)
        .then(|| archive_stem(&node.name))
        .flatten()
        .filter(|stem| is_folder(ground, &parent.join(stem)))
        .map(str::to_owned);
    Some(LargestRow {
        place: from_home(ground.home, parent),
        kind: item.kind,
        offer,
        unpacked,
        picked: ground.picked(&item.path),
        node,
    })
}

fn is_folder(ground: &Ground<'_>, path: &Path) -> bool {
    ground.state.find(path).and_then(|id| ground.state.node(id)).is_some_and(|node| node.kind == NodeKind::Folder)
}
