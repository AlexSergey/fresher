use std::path::Path;

use fresher_core::{EntryKind, FolderIdentity};

use crate::codec::decode::decode;
use crate::codec::fit::Limits;
use crate::tree::arena::Tree;
use crate::tree::node::{Flag, NodeIndex};
use crate::tree::testing::sums;

const MIB: u64 = 1 << 20;

/// No bound on the file: only the body limit counts.
const fn body_limit(body: usize) -> Limits {
    Limits { file: u64::MAX, body: body as u64 }
}

/// A root `/r` with a folder `f<i>` of `sizes[i]` MiB for each size, each with 20 subfolders that hold no bytes.
/// `mark` may change the tree (a hit, an unreadable folder) before it is frozen.
fn families(sizes: &[u64], mark: impl FnOnce(&mut Tree, &[Vec<NodeIndex>])) -> Tree {
    let mut tree = Tree::new();
    let root = tree.add_root(Path::new("/r"), FolderIdentity { device: 1, file_id: 2 }).unwrap();
    let mut children = Vec::new();
    for (position, &size) in sizes.iter().enumerate() {
        let family = tree.add_child(root, format!("f{position}").as_bytes(), EntryKind::Folder).unwrap();
        tree.add_own(family, sums(1, size * MIB, size * MIB, size * MIB)).unwrap();
        let mut kids = Vec::new();
        for child in 0..20 {
            let kid = tree.add_child(family, format!("child-{child:02}").as_bytes(), EntryKind::Folder).unwrap();
            tree.add_own(kid, sums(2, 10, 4096, 4096)).unwrap();
            kids.push(kid);
        }
        children.push(kids);
    }
    mark(&mut tree, &children);
    tree.freeze();
    tree
}

/// The names of the root's children that were collapsed, in name order.
fn collapsed_names(body: &[u8]) -> Vec<String> {
    let (tree, _) = decode(body).unwrap();
    let root = tree.roots()[0].node;
    tree.children(root)
        .filter(|&child| tree.node(child).unwrap().flags.has(Flag::Collapsed))
        .map(|child| String::from_utf8(tree.name(child).to_vec()).unwrap())
        .collect()
}

/// Every node of `fitted` has the totals of the node at its path in `original`.
fn assert_totals_kept(original: &Tree, fitted: &Tree) {
    for position in 0..fitted.len() {
        let index = NodeIndex::new(position).unwrap();
        let path = fitted.path(index).unwrap();
        let before = original.lookup(&path).unwrap();
        assert_eq!(fitted.sums(index), original.sums(before), "{}", path.display());
        assert_eq!(fitted.node(index).unwrap().folders, original.node(before).unwrap().folders);
        assert_eq!(fitted.completeness(index), original.completeness(before));
    }
}

mod negative {
    use crate::codec::CodecError;
    use crate::codec::decode::decode;
    use crate::codec::fit::tests::{body_limit, collapsed_names, families};
    use crate::codec::fit::{Limits, fit, fully_collapsed};
    use crate::codec::header::testing::header;
    use crate::tree::node::Hit;

    #[test]
    fn a_subtree_with_a_hit_is_never_collapsed() {
        let tree = families(&[1, 2, 3], |tree, children| {
            tree.add_hits(children[0][5], &[Hit { interest: 1, marker: None, siblings: 0 }]).unwrap();
        });
        let floor = fully_collapsed(&tree, &header());
        let body = fit(&tree, &header(), body_limit(floor.len())).unwrap();
        assert_eq!(collapsed_names(&body), ["f1", "f2"]);
        let (decoded, _) = decode(&body).unwrap();
        let kept = decoded.lookup("/r/f0/child-05".as_ref()).unwrap();
        assert_eq!(decoded.hits(kept).len(), 1);
    }

    #[test]
    fn a_subtree_with_an_unreadable_folder_is_never_collapsed() {
        let tree = families(&[1, 2, 3], |tree, children| {
            tree.mark_unreadable(children[1][3], fresher_core::Unreadable::NoPermission).unwrap();
        });
        let floor = fully_collapsed(&tree, &header());
        assert_eq!(collapsed_names(&fit(&tree, &header(), body_limit(floor.len())).unwrap()), ["f0", "f2"]);
    }

    #[test]
    fn a_tree_whose_hits_exceed_the_cap_is_not_saved() {
        let tree = families(&[1, 2], |tree, children| {
            for kids in children {
                tree.add_hits(kids[0], &[Hit { interest: 1, marker: None, siblings: 0 }]).unwrap();
            }
        });
        let floor = fully_collapsed(&tree, &header());
        assert!(matches!(fit(&tree, &header(), body_limit(floor.len() - 1)), Err(CodecError::TooLarge { .. })));
        assert!(matches!(
            fit(&tree, &header(), Limits { file: 100, body: u64::MAX }),
            Err(CodecError::TooLarge { .. })
        ));
    }
}

mod positive {
    use crate::codec::decode::decode;
    use crate::codec::encode::encode;
    use crate::codec::fit::tests::{assert_totals_kept, body_limit, collapsed_names, families};
    use crate::codec::fit::{Limits, fit, fully_collapsed, packed_len};
    use crate::codec::header::testing::header;
    use crate::codec::testing::{small_tree, wide_tree};
    use crate::tree::arena::Shared;
    use crate::tree::testing::sums;

    #[test]
    fn a_tree_under_the_cap_is_not_collapsed() {
        let tree = small_tree();
        let limits = Limits { file: 8 << 20, body: 64 << 20 };
        assert_eq!(fit(&tree, &header(), limits).unwrap(), encode(&tree, &header()).unwrap());
    }

    #[test]
    fn a_body_over_the_cap_that_fits_compressed_is_not_collapsed() {
        let tree = wide_tree(50_000);
        let body = encode(&tree, &header()).unwrap();
        let file = body.len() as u64 / 2;
        assert!(packed_len(&body) + 64 <= file, "the body compresses to under half");
        assert_eq!(fit(&tree, &header(), Limits { file, body: u64::MAX }).unwrap(), body);
    }

    #[test]
    fn the_smallest_subtrees_collapse_first() {
        let tree = families(&[3, 1, 4, 2], |_, _| {});
        let full = encode(&tree, &header()).unwrap().len();
        let floor = fully_collapsed(&tree, &header()).len();
        let by_size = ["f1", "f3", "f0", "f2"];
        let mut seen = 0;
        for limit in (floor..full).rev().step_by(16) {
            let mut names = collapsed_names(&fit(&tree, &header(), body_limit(limit)).unwrap());
            assert!(names.len() >= seen, "a smaller cap never collapses less");
            seen = names.len();
            let mut expected = by_size[..names.len()].to_vec();
            expected.sort_unstable();
            names.sort_unstable();
            assert_eq!(names, expected, "at a cap of {limit}");
        }
        assert_eq!(seen, 4);
    }

    #[test]
    fn collapsing_keeps_every_total() {
        let tree = families(&[1, 2, 3], |tree, children| {
            let shared = sums(0, 5 * 1024, 8192, 8192);
            tree.add_shared(Shared::HardLink, 1, 40, children[2][1], shared).unwrap();
            tree.add_shared(Shared::HardLink, 1, 40, children[0][0], shared).unwrap();
            tree.add_shared(Shared::HardLink, 1, 41, children[1][9], shared).unwrap();
            tree.add_shared(Shared::HardLink, 1, 41, children[1][2], shared).unwrap();
            tree.add_shared(Shared::Clone, 1, 7, children[2][4], sums(0, 0, 3 * 4096, 0)).unwrap();
        });
        let floor = fully_collapsed(&tree, &header());
        let (decoded, _) = decode(&fit(&tree, &header(), body_limit(floor.len())).unwrap()).unwrap();
        assert_eq!(decoded.len(), 4, "the root and three collapsed folders");
        assert_totals_kept(&tree, &decoded);
    }
}

mod property {
    use proptest::prelude::*;

    use crate::codec::CodecError;
    use crate::codec::decode::decode;
    use crate::codec::encode::encode;
    use crate::codec::fit::tests::{assert_totals_kept, body_limit};
    use crate::codec::fit::{Limits, fit, fully_collapsed, packed_len};
    use crate::codec::header::testing::header;
    use crate::codec::testing::{build, node_specs};

    proptest! {
        #[test]
        fn the_fitted_body_is_never_over_the_cap((two_roots, specs) in node_specs(150), share in 0..=1000_usize) {
            let tree = build(two_roots, &specs);
            let full = encode(&tree, &header()).unwrap().len();
            let floor = fully_collapsed(&tree, &header()).len();
            let limit = floor + (full - floor) * share / 1000;
            let body = fit(&tree, &header(), body_limit(limit)).unwrap();
            prop_assert!(body.len() <= limit);
            assert_totals_kept(&tree, &decode(&body).unwrap().0);
            let refused = matches!(fit(&tree, &header(), body_limit(floor - 1)), Err(CodecError::TooLarge { .. }));
            prop_assert!(refused);
            let file = packed_len(&fully_collapsed(&tree, &header())) + 64;
            let body = fit(&tree, &header(), Limits { file, body: u64::MAX }).unwrap();
            prop_assert!(packed_len(&body) + 64 <= file);
        }
    }
}
