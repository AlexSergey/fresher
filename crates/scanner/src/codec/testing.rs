//! Test helpers of the codec: a small tree with every kind of node, random trees, and a snapshot that holds everything
//! a body must keep, so two trees compare equal whatever their arena order.

use std::path::{Path, PathBuf};

use fresher_core::{EntryAccess, EntryKind, FolderIdentity, Inside, MarkerCheck, Timestamp, Unreadable};
use proptest::prelude::*;

use crate::codec::encode::GROUP_KINDS;
use crate::tree::access::NodeAccess;
use crate::tree::arena::{Shared, Tree};
use crate::tree::node::{Flag, Hit, NodeIndex, Sums};
use crate::tree::testing::{Shape, canonical, sums};
use crate::tree::times::{Stamp, Times};

/// Everything a body keeps of a tree: the canonical nodes, each node's time, owner, access and stamp, the roots'
/// identities, and every group with the paths of its members and the bytes it counts once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub(crate) shapes: Vec<Shape>,
    pub(crate) per_node: Vec<(Option<Timestamp>, u32, NodeAccess, Stamp)>,
    pub(crate) identities: Vec<FolderIdentity>,
    pub(crate) groups: Vec<(usize, u64, u64, Vec<PathBuf>, Sums)>,
}

pub(crate) fn snapshot(tree: &Tree) -> Snapshot {
    let mut per_node = Vec::new();
    let mut stack: Vec<NodeIndex> = tree.roots().iter().rev().map(|root| root.node).collect();
    while let Some(index) = stack.pop() {
        let node = tree.node(index).unwrap();
        per_node.push((node.modified(), node.owner, node.access, tree.stamp(index)));
        let children: Vec<_> = tree.children(index).collect();
        stack.extend(children.into_iter().rev());
    }
    let mut groups = Vec::new();
    for (slot, kind) in GROUP_KINDS.into_iter().enumerate() {
        let mut members: Vec<_> = tree.shared(kind).to_vec();
        members.sort_by_key(|member| (member.device, member.id));
        for group in members.chunk_by(|left, right| (left.device, left.id) == (right.device, right.id)) {
            let mut paths: Vec<PathBuf> = group.iter().map(|member| tree.path(member.holder).unwrap()).collect();
            paths.sort();
            let bytes = group.iter().fold(Sums::ZERO, |most, member| Sums {
                files: most.files.max(member.bytes.files),
                logical: most.logical.max(member.bytes.logical),
                allocated: most.allocated.max(member.bytes.allocated),
                reclaimable: most.reclaimable.max(member.bytes.reclaimable),
            });
            groups.push((slot, group[0].device, group[0].id, paths, bytes));
        }
    }
    Snapshot {
        shapes: canonical(tree),
        per_node,
        identities: tree.roots().iter().map(|root| root.identity).collect(),
        groups,
    }
}

const MIB: u64 = 1 << 20;

/// A home-like tree with every kind of node: folders, a large kept file, a matched link, hits with marker checks
/// and siblings, an owner, unreadable, avoided and opaque folders, a hard link group across two folders and a
/// clone group. Frozen; its owners index the header's two claims.
pub(crate) fn small_tree() -> Tree {
    let mut tree = Tree::new();
    let home = tree.add_root(Path::new("/home/user"), FolderIdentity { device: 16, file_id: 2 }).unwrap();
    tree.set_modified(home, Some(Timestamp::from_secs(1_763_000_000))).unwrap();
    tree.set_access(home, access(0o755, false)).unwrap();
    tree.add_own(home, sums(4, 9_000, 16_384, 16_384)).unwrap();
    let library = folder(&mut tree, home, "Library", sums(1, 100, 4096, 4096));
    let caches = folder(&mut tree, library, "Caches", Sums::ZERO);
    for (name, size) in [("com.example.a", 10), ("com.example.b", 20), ("com.example.app", 30)] {
        let cache = folder(&mut tree, caches, name, sums(5, size * MIB, size * MIB, size * MIB));
        tree.set_modified(cache, Some(Timestamp::from_secs(1_763_000_100 + size))).unwrap();
    }
    let mail = folder(&mut tree, library, "Mail", Sums::ZERO);
    tree.mark_unreadable(mail, Unreadable::NoPermission).unwrap();
    let projects = folder(&mut tree, home, "projects", sums(2, 500, 8192, 8192));
    let app = folder(&mut tree, projects, "app", sums(3, 900, 12_288, 12_288));
    let modules = folder(&mut tree, app, "node_modules", Sums::ZERO);
    tree.add_hits(modules, &[Hit { interest: 3, marker: None, siblings: 0b101 }]).unwrap();
    let at = |secs| Some(Timestamp::from_secs(secs));
    tree.set_times(modules, Times { changed: at(1_763_000_200), accessed: None }).unwrap();
    let inside = Inside { newest_change: at(1_763_000_900), newest_file_access: at(1_762_000_000), ..Inside::EMPTY };
    tree.set_inside(modules, Some(Inside { foreign: true, ..inside })).unwrap();
    tree.apply_opaque(modules, sums(812, 40 * MIB, 48 * MIB, 48 * MIB), 97, &[]).unwrap();
    let target = folder(&mut tree, app, "target", Sums::ZERO);
    tree.add_hits(target, &[Hit { interest: 5, marker: Some(MarkerCheck::Valid), siblings: 1 }]).unwrap();
    tree.set_access(target, access(0o555, false)).unwrap();
    let video = tree.add_child(home, b"video.mov", EntryKind::File).unwrap();
    tree.set_access(video, access(0o444, true)).unwrap();
    tree.add_own(video, sums(0, 700 * MIB, 700 * MIB, 700 * MIB)).unwrap();
    let link = tree.add_child(app, b"app.log", EntryKind::Symlink).unwrap();
    tree.add_hits(link, &[Hit { interest: 7, marker: Some(MarkerCheck::Invalid), siblings: 0 }]).unwrap();
    tree.set_times(link, Times { changed: at(1_763_000_300), accessed: at(1_763_000_400) }).unwrap();
    let documents = folder(&mut tree, home, "Documents", Sums::ZERO);
    tree.mark(documents, Flag::Avoided).unwrap();
    let shared = sums(0, 64 * 1024, 64 * 1024, 64 * 1024);
    tree.add_shared(Shared::HardLink, 16, 900, projects, shared).unwrap();
    tree.add_shared(Shared::HardLink, 16, 900, caches, shared).unwrap();
    tree.add_shared(Shared::Clone, 16, 77, app, sums(0, 0, 3 * MIB, 0)).unwrap();
    tree.add_shared(Shared::Clone, 16, 77, video, sums(0, 0, 2 * MIB, 0)).unwrap();
    tree.freeze();
    tree.set_owner(modules, 0).unwrap();
    tree.set_owner(target, 1).unwrap();
    tree
}

/// The access of the user's own entry of `mode`, locked by the user when `locked`.
fn access(mode: u16, locked: bool) -> NodeAccess {
    NodeAccess::of(EntryAccess { owner: Some(501), mode, user_immutable: locked, ..EntryAccess::default() }, Some(501))
}

fn folder(tree: &mut Tree, parent: NodeIndex, name: &str, own: Sums) -> NodeIndex {
    let index = tree.add_child(parent, name.as_bytes(), EntryKind::Folder).unwrap();
    tree.add_own(index, own).unwrap();
    index
}

/// A tree of `count` folders under one root, with the name shapes of the tree's heap test (4 to 20 bytes, 12 on
/// average, eight children a folder) and the same folded bytes in each.
pub(crate) fn wide_tree(count: usize) -> Tree {
    let mut tree = Tree::new();
    let root = tree.add_root(Path::new("/home/user"), FolderIdentity { device: 1, file_id: 2 }).unwrap();
    let mut folders = vec![root];
    let mut name = Vec::new();
    while tree.len() < count {
        let parent = folders[(tree.len() - 1) / 8];
        name.clear();
        name.resize(4 + tree.len() % 17, b'n');
        name.extend_from_slice(tree.len().to_string().as_bytes());
        let child = tree.add_child(parent, &name, EntryKind::Folder).unwrap();
        tree.add_own(child, sums(3, 1000, 4096, 4096)).unwrap();
        folders.push(child);
    }
    tree.freeze();
    tree
}

/// One node of a random tree.
#[derive(Clone, Debug)]
pub(crate) struct NodeSpec {
    /// Which earlier folder (or root) holds it.
    parent: usize,
    /// 0 a folder, 1 a kept file, 2 a matched link, 3 a matched socket, pipe or device.
    kind: u8,
    own: Sums,
    modified: Option<u32>,
    hits: Vec<(u32, u8, u64)>,
    owner: Option<u8>,
    /// The bits of its access.
    access: u8,
    /// 0 read, 1 unreadable, 2 avoided, 3 an entry unreadable, 4 opaque (when it has no children), 5 another volume.
    state: u8,
    /// A group it belongs to: a hard link group when true, else a clone group; the group's id; the bytes.
    shared: Option<(bool, u8, u32)>,
    /// Its status change, its access and its inside; kept only when it has hits.
    stamp: (Option<u32>, Option<u32>, Option<InsideSpec>),
}

/// An inside: its newest change, its newest file access, the bits of its facts (special, foreign, complete,
/// a lock file's name).
type InsideSpec = (Option<u32>, Option<u32>, u8);

/// Up to `max` random nodes under one or two roots.
pub(crate) fn node_specs(max: usize) -> impl Strategy<Value = (bool, Vec<NodeSpec>)> {
    let own = (0..1000_u32, 0..1_u64 << 36, 0..1_u64 << 36, 0..1_u64 << 36)
        .prop_map(|(files, logical, allocated, reclaimable)| sums(files, logical, allocated, reclaimable));
    let hits = prop::collection::vec((0..64_u32, 0..3_u8, any::<u64>()), 0..3);
    let shared = prop::option::weighted(0.3, (any::<bool>(), 0..6_u8, 1..1_000_000_u32));
    let time = || prop::option::of(1_600_000_000..1_900_000_000_u32);
    let stamp = (time(), time(), prop::option::of((time(), time(), 0..16_u8)));
    let node = (
        any::<usize>(),
        0..4_u8,
        own,
        prop::option::of(1_700_000_000..1_800_000_000_u32),
        prop::option::weighted(0.2, hits),
        prop::option::weighted(0.1, 0..3_u8),
        0..32_u8,
        prop::sample::select(vec![0_u8, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5]),
        shared,
        stamp,
    )
        .prop_map(|(parent, kind, own, modified, hits, owner, access, state, shared, stamp)| NodeSpec {
            parent,
            kind,
            own,
            modified,
            hits: hits.unwrap_or_default(),
            owner,
            access,
            state,
            shared,
            stamp,
        });
    (any::<bool>(), prop::collection::vec(node, 0..max))
}

/// The frozen tree of `specs`; `two_roots` adds a second root. Owners index a table of three claims.
pub(crate) fn build(two_roots: bool, specs: &[NodeSpec]) -> Tree {
    let mut tree = Tree::new();
    let mut folders = vec![tree.add_root(Path::new("/r"), FolderIdentity { device: 1, file_id: 2 }).unwrap()];
    if two_roots {
        folders.push(tree.add_root(Path::new("/Volumes/x"), FolderIdentity { device: 3, file_id: 4 }).unwrap());
    }
    let mut nodes = Vec::with_capacity(specs.len());
    for (position, spec) in specs.iter().enumerate() {
        let parent = folders[spec.parent % folders.len()];
        let prefix = ["n", "Library", "node_modules", "a b"][position % 4];
        let name = format!("{prefix}{position}");
        let kind = [EntryKind::Folder, EntryKind::File, EntryKind::Symlink, EntryKind::Other][usize::from(spec.kind)];
        let index = tree.add_child(parent, name.as_bytes(), kind).unwrap();
        let own = if kind == EntryKind::Folder { spec.own } else { Sums { files: 0, ..spec.own } };
        tree.add_own(index, own).unwrap();
        tree.set_modified(index, spec.modified.map(|secs| Timestamp::from_secs(u64::from(secs)))).unwrap();
        tree.set_access(index, NodeAccess::from_bits(spec.access).unwrap()).unwrap();
        let hits: Vec<Hit> = spec
            .hits
            .iter()
            .map(|&(interest, marker, siblings)| Hit {
                interest,
                marker: [None, Some(MarkerCheck::Valid), Some(MarkerCheck::Invalid)][usize::from(marker)],
                siblings,
            })
            .collect();
        tree.add_hits(index, &hits).unwrap();
        if !hits.is_empty() {
            let at = |secs: Option<u32>| secs.map(|secs| Timestamp::from_secs(u64::from(secs)));
            let (changed, accessed, inside) = spec.stamp;
            tree.set_times(index, Times { changed: at(changed), accessed: at(accessed) }).unwrap();
            let inside = inside.map(|(change, access, bits)| Inside {
                newest_change: at(change),
                newest_file_access: at(access),
                special: bits & 1 != 0,
                foreign: bits & 2 != 0,
                complete: bits & 4 != 0,
                lock_named: bits & 8 != 0,
            });
            tree.set_inside(index, inside).unwrap();
        }
        if let Some((link, id, bytes)) = spec.shared {
            let (kind, bytes) = if link {
                (Shared::HardLink, sums(0, u64::from(bytes), u64::from(bytes), u64::from(bytes)))
            } else {
                (Shared::Clone, sums(0, 0, u64::from(bytes), 0))
            };
            tree.add_shared(kind, 1, u64::from(id), index, bytes).unwrap();
        }
        if kind == EntryKind::Folder {
            folders.push(index);
        }
        nodes.push(index);
    }
    for (spec, &index) in specs.iter().zip(&nodes) {
        if spec.kind != 0 {
            continue;
        }
        match spec.state {
            1 => tree.mark_unreadable(index, Unreadable::Guarded).unwrap(),
            2 => tree.mark(index, Flag::Avoided).unwrap(),
            3 => tree.mark(index, Flag::Incomplete).unwrap(),
            4 if tree.children(index).next().is_none() => {
                tree.add_hidden_folders(index, spec.own.files % 50).unwrap();
                tree.mark(index, Flag::Opaque).unwrap();
            }
            5 => tree.mark(index, Flag::OtherVolume).unwrap(),
            _ => {}
        }
    }
    tree.freeze();
    for (spec, &index) in specs.iter().zip(&nodes) {
        if let Some(owner) = spec.owner {
            tree.set_owner(index, u32::from(owner)).unwrap();
        }
    }
    tree
}
