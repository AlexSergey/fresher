//! Tests of the codec: round trips, every corruption a file can suffer, and the size of a body.

use crate::codec::header::Header;
use crate::codec::header::testing::header;
use crate::codec::varint::write_u64;

/// A body written by hand: the test header, then one root `/r` with `children` declared and `nodes` in all, and
/// `records` after it (each a complete child record).
fn handmade(children: u64, nodes: u64, records: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    Header::write(&header(), &mut out);
    write_u64(&mut out, 1);
    write_u64(&mut out, nodes);
    out.extend_from_slice(&[0, 2, b'/', b'r', 1]);
    write_u64(&mut out, children);
    out.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2]);
    for record in records {
        out.extend_from_slice(record);
    }
    out.extend_from_slice(&[0, 0]);
    out
}

/// A kept file's record: the name front-coded against `shared` bytes of the previous sibling.
fn file_record(shared: u8, rest: &[u8]) -> Vec<u8> {
    let mut record = vec![shared, u8::try_from(rest.len()).unwrap()];
    record.extend_from_slice(rest);
    record.extend_from_slice(&[2, 1, 0, 0, 0, 0, 0, 0, 0]);
    record
}

mod negative {
    use crate::codec::CodecError;
    use crate::codec::decode::decode;
    use crate::codec::encode::encode;
    use crate::codec::header::testing::header;
    use crate::codec::testing::small_tree;
    use crate::codec::tests::{file_record, handmade};
    use crate::tree::testing::rooted;

    #[test]
    fn a_body_cut_anywhere_is_refused() {
        let body = encode(&small_tree(), &header()).unwrap();
        for len in 0..body.len() {
            assert!(decode(&body[..len]).is_err(), "a body cut at {len} of {} was accepted", body.len());
        }
    }

    #[test]
    fn a_child_count_larger_than_the_rest_is_refused() {
        let file = file_record(0, b"a");
        assert!(decode(&handmade(1, 2, &[&file])).is_ok());
        assert_eq!(decode(&handmade(2, 2, &[&file])).err(), Some(CodecError::CountTooLarge));
        assert_eq!(decode(&handmade(1_000_000, 2, &[&file])).err(), Some(CodecError::CountTooLarge));
        assert_eq!(decode(&handmade(1, 1_000_000, &[&file])).err(), Some(CodecError::CountTooLarge));
    }

    #[test]
    fn another_tree_format_is_refused() {
        let mut body = encode(&small_tree(), &header()).unwrap();
        body[0] = body[0].wrapping_add(1);
        assert!(matches!(decode(&body), Err(CodecError::OtherFormat { .. })));
    }

    #[test]
    fn a_body_of_tree_format_5_is_refused() {
        let mut body = encode(&small_tree(), &header()).unwrap();
        body[..2].copy_from_slice(&5_u16.to_le_bytes());
        assert_eq!(decode(&body).err(), Some(CodecError::OtherFormat { found: 5 }));
    }

    #[test]
    fn an_overlong_varint_is_refused() {
        let mut body = handmade(1, 2, &[&file_record(0, b"a")]);
        let at = body.len() - 2;
        body.splice(at..=at, [0x80, 0x00]);
        assert_eq!(decode(&body).err(), Some(CodecError::OverlongNumber));
    }

    #[test]
    fn a_name_that_is_not_one_plain_name_is_refused() {
        for name in [&b""[..], b".", b"..", b"a/b", b"a\0b"] {
            assert_eq!(decode(&handmade(1, 2, &[&file_record(0, name)])).err(), Some(CodecError::BadName), "{name:?}");
        }
    }

    #[test]
    fn siblings_out_of_order_or_twice_are_refused() {
        let (first, again, before) = (file_record(0, b"b"), file_record(1, b""), file_record(0, b"a"));
        assert_eq!(decode(&handmade(2, 3, &[&first, &again])).err(), Some(CodecError::BadName));
        assert_eq!(decode(&handmade(2, 3, &[&first, &before])).err(), Some(CodecError::BadName));
        assert_eq!(decode(&handmade(1, 2, &[&file_record(3, b"a")])).err(), Some(CodecError::BadName));
    }

    #[test]
    fn a_folder_flag_on_a_file_is_refused() {
        let mut record = file_record(0, b"a");
        record[3] = 2 | 1 << 4;
        assert_eq!(decode(&handmade(1, 2, &[&record])).err(), Some(CodecError::BadFlags));
        record[3] = 2 | 1;
        assert_eq!(decode(&handmade(1, 2, &[&record])).err(), Some(CodecError::BadFlags));
    }

    #[test]
    fn an_owner_outside_the_claims_is_refused() {
        let mut record = file_record(0, b"a");
        let owner = record.len() - 2;
        record[owner] = 4;
        assert_eq!(decode(&handmade(1, 2, &[&record])).err(), Some(CodecError::UnknownOwner));
        let mut tree = small_tree();
        tree.set_owner(tree.roots()[0].node, 3).unwrap();
        assert_eq!(encode(&tree, &header()).err(), Some(CodecError::UnknownOwner));
    }

    #[test]
    fn an_access_with_a_bit_no_fact_uses_is_refused() {
        let mut record = file_record(0, b"a");
        *record.last_mut().unwrap() = 1 << 5;
        assert_eq!(decode(&handmade(1, 2, &[&record])).err(), Some(CodecError::BadFlags));
    }

    #[test]
    fn bytes_after_the_body_are_refused() {
        let mut body = encode(&small_tree(), &header()).unwrap();
        body.push(0);
        assert_eq!(decode(&body).err(), Some(CodecError::TrailingBytes));
    }

    #[test]
    fn a_tree_that_is_not_frozen_is_not_written() {
        let (tree, _) = rooted("/r");
        assert_eq!(encode(&tree, &header()).err(), Some(CodecError::NotFrozen));
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{ByteSize, EntryKind, FolderIdentity, ScanCache, ScopeKey};
    use fresher_testing::MemoryScanCache;

    use crate::codec::decode::decode;
    use crate::codec::encode::encode;
    use crate::codec::header::testing::header;
    use crate::codec::testing::{small_tree, snapshot, wide_tree};
    use crate::codec::tests::{file_record, handmade};
    use crate::tree::access::NodeAccess;
    use crate::tree::arena::Tree;
    use crate::tree::node::Hit;
    use crate::tree::testing::{canonical, rooted};

    #[test]
    fn a_tree_survives_a_round_trip() {
        let tree = small_tree();
        let body = encode(&tree, &header()).unwrap();
        let (decoded, read) = decode(&body).unwrap();
        assert_eq!(read, header());
        assert_eq!(snapshot(&decoded), snapshot(&tree));
        assert_eq!(encode(&decoded, &header()).unwrap(), body, "a second write gives the same bytes");
    }

    #[test]
    fn the_held_names_of_every_hit_survive_a_round_trip() {
        let tree = small_tree();
        let (decoded, _) = decode(&encode(&tree, &header()).unwrap()).unwrap();
        let held = |tree: &Tree| {
            canonical(tree).iter().flat_map(|shape| shape.hits.iter().map(|hit| hit.holds)).collect::<Vec<_>>()
        };
        assert_eq!(held(&decoded), held(&tree));
        assert!(held(&tree).contains(&0b110) && held(&tree).contains(&(1 << 63)));
    }

    #[test]
    fn every_nodes_access_survives_a_round_trip() {
        let tree = small_tree();
        let (decoded, _) = decode(&encode(&tree, &header()).unwrap()).unwrap();
        let access = |tree: &Tree| snapshot(tree).per_node.iter().map(|&(_, _, access, _)| access).collect::<Vec<_>>();
        assert_eq!(access(&decoded), access(&tree));
        assert!(access(&tree).iter().filter(|&&access| access != NodeAccess::NONE).count() >= 3);
    }

    #[test]
    fn the_times_and_the_inside_of_a_node_with_hits_survive_a_round_trip() {
        let tree = small_tree();
        let (decoded, _) = decode(&encode(&tree, &header()).unwrap()).unwrap();
        let stamps = |tree: &Tree| snapshot(tree).per_node.iter().map(|&(_, _, _, stamp)| stamp).collect::<Vec<_>>();
        assert_eq!(stamps(&decoded), stamps(&tree));
        assert!(stamps(&tree).iter().any(|stamp| stamp.inside.is_some_and(|inside| inside.foreign)));
        assert!(stamps(&tree).iter().any(|stamp| stamp.times.accessed.is_some()));
    }

    #[test]
    fn a_socket_a_pipe_or_a_device_keeps_its_kind_through_a_round_trip() {
        let (mut tree, root) = rooted("/r");
        let pipe = tree.add_child(root, b"pipe", EntryKind::Other).unwrap();
        tree.add_hits(pipe, &[Hit { interest: 0, marker: None, siblings: 0, holds: 0 }]).unwrap();
        tree.freeze();

        let (decoded, _) = decode(&encode(&tree, &header()).unwrap()).unwrap();

        let root = decoded.roots()[0].node;
        let kinds: Vec<_> = decoded.children(root).map(|child| decoded.node(child).unwrap().flags.kind()).collect();
        assert_eq!(kinds, [EntryKind::Other]);
        assert_eq!(snapshot(&decoded), snapshot(&tree));
    }

    #[test]
    fn a_handmade_body_reads_as_written() {
        let (tree, _) = decode(&handmade(2, 3, &[&file_record(0, b"ab"), &file_record(1, b"c")])).unwrap();
        let root = tree.roots()[0].node;
        let names: Vec<&[u8]> = tree.children(root).map(|child| tree.name(child)).collect();
        assert_eq!(names, [&b"ab"[..], b"ac"]);
        assert_eq!(tree.sums(root).unwrap().files, 2);
    }

    #[test]
    fn front_coding_restores_every_name() {
        let mut tree = Tree::new();
        let root = tree.add_root(Path::new("/r"), FolderIdentity { device: 1, file_id: 2 }).unwrap();
        let names: [&[u8]; 7] =
            [b"Libraries", b"Library", b"Library Caches", b"Library-1", b"a", b"com.example.app", b"\xff\xfe raw"];
        for name in names {
            tree.add_child(root, name, EntryKind::Folder).unwrap();
        }
        tree.freeze();
        let body = encode(&tree, &header()).unwrap();
        let (decoded, _) = decode(&body).unwrap();
        let root = decoded.roots()[0].node;
        let read: Vec<&[u8]> = decoded.children(root).map(|child| decoded.name(child)).collect();
        assert_eq!(read, names);
        assert_eq!(body.windows(5).filter(|window| window == b"Libra").count(), 1, "the shared prefix is written once");
    }

    #[test]
    fn an_encoded_tree_is_accepted_by_the_memory_cache() {
        let cache = MemoryScanCache::with_cap(ByteSize::new(8 << 20));
        let body = encode(&small_tree(), &header()).unwrap();
        cache.save(ScopeKey::new(1), &body).unwrap();
        let loaded = cache.load(ScopeKey::new(1)).unwrap().unwrap();
        assert_eq!(snapshot(&decode(&loaded).unwrap().0), snapshot(&small_tree()));
    }

    #[test]
    fn cache_bytes_per_folder_node_stay_under_24() {
        const NODES: usize = 100_000;
        let tree = wide_tree(NODES);
        let body = encode(&tree, &header()).unwrap();
        let per_node = body.len() / NODES;
        assert!(per_node < 24, "{} bytes for {NODES} folder nodes", body.len());
        assert_eq!(decode(&body).unwrap().0.len(), NODES);
    }
}

mod property {
    use proptest::prelude::*;

    use crate::codec::decode::decode;
    use crate::codec::encode::encode;
    use crate::codec::header::testing::header;
    use crate::codec::testing::{build, node_specs, small_tree, snapshot};

    proptest! {
        #[test]
        fn any_tree_round_trips((two_roots, specs) in node_specs(120)) {
            let tree = build(two_roots, &specs);
            let body = encode(&tree, &header()).unwrap();
            let (decoded, read) = decode(&body).unwrap();
            prop_assert_eq!(read, header());
            prop_assert_eq!(snapshot(&decoded), snapshot(&tree));
            prop_assert_eq!(encode(&decoded, &header()).unwrap(), body);
        }

        #[test]
        fn decode_never_panics_on_any_bytes(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
            let _ = decode(&bytes);
        }

        #[test]
        fn decode_never_panics_on_a_damaged_body(at in any::<usize>(), byte in any::<u8>()) {
            let mut body = encode(&small_tree(), &header()).unwrap();
            let at = at % body.len();
            body[at] = byte;
            if let Ok((tree, read)) = decode(&body) {
                prop_assert!(encode(&tree, &read).is_ok());
            }
        }
    }
}
