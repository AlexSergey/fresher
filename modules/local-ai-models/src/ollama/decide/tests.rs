//! The store's and the models' rows on hand-made matches and manifests: every row that keeps first, then the offer.

use std::collections::BTreeMap;
use std::path::PathBuf;

use fresher_core::{ByteSize, EntryFacts, EntryKind, Sizes, Timestamp};

use crate::gathered::Candidate;
use crate::ollama::decide::{Entries, ModelVerdict, StoreVerdict, model, store};
use crate::ollama::manifest::{Manifest, Unreadable};
use crate::ollama::test_support::{digest, in_store, listed, manifest, manifest_path};
use crate::stores::OllamaInterest;

const CHANGED: u64 = 1_760_000_000;
const ACCESSED: u64 = 1_765_000_000;
/// Each manifest's logical size.
const MANIFEST: u64 = 500;

/// A match of `which` at `path`, the user's, with its own times.
fn matched(which: OllamaInterest, path: PathBuf, kind: EntryKind, bytes: u64) -> Candidate {
    Candidate {
        row: 3,
        ollama: Some(which),
        path,
        kind,
        sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes.next_multiple_of(4096))),
        modified: Some(Timestamp::from_secs(CHANGED)),
        accessed: Some(Timestamp::from_secs(ACCESSED)),
        facts: EntryFacts::REMOVABLE,
        holds: Vec::new(),
        inside: None,
    }
}

/// The layer of the digest of `byte`, of its listed size.
fn layer(byte: u8) -> Candidate {
    matched(
        OllamaInterest::Blobs,
        in_store(&format!("blobs/{}", digest(byte).blob_name())),
        EntryKind::File,
        listed(byte),
    )
}

/// A store's matches and the manifest each of its manifest files holds.
#[derive(Clone, Debug)]
struct World {
    candidates: Vec<Candidate>,
    manifests: BTreeMap<PathBuf, Result<Manifest, Unreadable>>,
}

impl World {
    /// The store holding its layout, its registry's folders, and no model.
    fn empty() -> Self {
        let mut store = matched(OllamaInterest::Store, in_store(""), EntryKind::Folder, 0);
        store.holds = vec!["manifests".into(), "blobs".into()];
        let folders = [
            (OllamaInterest::Hosts, "manifests/registry.ollama.ai"),
            (OllamaInterest::Namespaces, "manifests/registry.ollama.ai/library"),
        ];
        let mut candidates = vec![store];
        candidates.extend(folders.map(|(which, place)| matched(which, in_store(place), EntryKind::Folder, 0)));
        Self { candidates, manifests: BTreeMap::new() }
    }

    /// `llama3:latest` and `llama3:8b` (one model of config 1 and layers 2, 3), `mistral:7b` (config 4, layer 5) and
    /// `chat:1`, made from mistral (config 6, layers 5 and 7), every layer there.
    fn mixed() -> Self {
        Self::empty()
            .with_model("llama3", "latest", 1, &[2, 3])
            .with_model("llama3", "8b", 1, &[2, 3])
            .with_model("mistral", "7b", 4, &[5])
            .with_model("chat", "1", 6, &[5, 7])
            .with_layers(&[1, 2, 3, 4, 5, 6, 7])
    }

    /// With the manifest `model:tag` of `config` and `layers`, and its model's folder.
    fn with_model(mut self, model: &str, tag: &str, config: u8, layers: &[u8]) -> Self {
        let folder = in_store(&format!("manifests/registry.ollama.ai/library/{model}"));
        if !self.candidates.iter().any(|candidate| candidate.path == folder) {
            self.candidates.push(matched(OllamaInterest::Models, folder, EntryKind::Folder, 0));
        }
        let path = manifest_path(model, tag);
        self.candidates.push(matched(OllamaInterest::Manifests, path.clone(), EntryKind::File, MANIFEST));
        self.manifests.insert(path, Ok(manifest(config, layers)));
        self
    }

    fn with_layers(mut self, bytes: &[u8]) -> Self {
        self.candidates.extend(bytes.iter().map(|byte| layer(*byte)));
        self
    }

    fn with(mut self, candidate: Candidate) -> Self {
        self.candidates.push(candidate);
        self
    }

    /// With the match at `path` changed by `change`.
    fn changed(mut self, path: &PathBuf, change: impl FnOnce(&mut Candidate)) -> Self {
        change(self.candidates.iter_mut().find(|candidate| &candidate.path == path).unwrap());
        self
    }

    fn without(mut self, path: &PathBuf) -> Self {
        self.candidates.retain(|candidate| &candidate.path != path);
        self
    }

    /// The store's verdict and, when it is open, every model's in name order.
    fn decide(&self) -> (StoreVerdict, Vec<ModelVerdict>) {
        let entries = Entries::new(&self.candidates);
        let read = entries.manifests().iter().map(|(_, candidate)| self.manifests[&candidate.path].clone()).collect();
        let verdict = store(&entries, read);
        let models = match &verdict {
            StoreVerdict::Open(layout) => (0..layout.models().len()).map(|at| model(&entries, layout, at)).collect(),
            _ => Vec::new(),
        };
        (verdict, models)
    }

    /// The store's sizes: its manifests and its layers.
    fn store_sizes(&self) -> Sizes {
        Entries::new(&self.candidates).sizes()
    }
}

fn foreign(candidate: &mut Candidate) {
    candidate.facts = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
}

fn logical(verdict: &ModelVerdict) -> Option<u64> {
    match verdict {
        ModelVerdict::LeftOut(_, sizes) => Some(sizes.logical.bytes()),
        ModelVerdict::Offered(offer) => Some(offer.freed.logical.bytes()),
        ModelVerdict::NotClaimed => None,
    }
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, MAX_PARTS};

    use super::{MANIFEST, World, foreign, layer, logical, matched};
    use crate::decide::Reason;
    use crate::ollama::decide::{ModelVerdict, StoreVerdict};
    use crate::ollama::manifest::Unreadable;
    use crate::ollama::test_support::{digest, in_store, listed, manifest_path};
    use crate::stores::OllamaInterest;

    #[test]
    fn no_store_or_no_layout_is_not_claimed() {
        let store = in_store("");

        let gone = World::mixed().without(&store);
        let no_blobs = World::mixed().changed(&store, |found| found.holds = vec!["manifests".into()]);
        let no_manifests = World::mixed().changed(&store, |found| found.holds = vec!["blobs".into()]);

        for world in [gone, no_blobs, no_manifests] {
            assert_eq!(world.decide(), (StoreVerdict::NotClaimed, Vec::new()));
        }
    }

    #[test]
    fn a_foreign_manifests_folder_leaves_out_the_store() {
        for folder in [
            "manifests/registry.ollama.ai",
            "manifests/registry.ollama.ai/library",
            "manifests/registry.ollama.ai/library/mistral",
        ] {
            let world = World::mixed().changed(&in_store(folder), foreign);

            assert_eq!(world.decide().0, StoreVerdict::LeftOut(Reason::NotYours, world.store_sizes()), "{folder}");
        }
    }

    #[test]
    fn a_foreign_layer_leaves_out_the_store() {
        let layer_of_mistral = in_store(&format!("blobs/{}", digest(5).blob_name()));
        let locked = |found: &mut EntryFacts| found.user_immutable = true;

        let theirs = World::mixed().changed(&layer_of_mistral, foreign);
        let mine_locked = World::mixed().changed(&layer_of_mistral, |found| locked(&mut found.facts));
        let manifest = World::mixed().changed(&manifest_path("chat", "1"), |found| {
            found.facts = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        });

        for world in [theirs, mine_locked, manifest] {
            assert_eq!(world.decide(), (StoreVerdict::LeftOut(Reason::NotYours, world.store_sizes()), Vec::new()));
        }
    }

    #[test]
    fn a_partial_download_leaves_out_the_store() {
        for name in [format!("{}-partial", digest(9).blob_name()), format!("{}-partial-3", digest(9).blob_name())] {
            let world = World::mixed().with(matched(
                OllamaInterest::Blobs,
                in_store(&format!("blobs/{name}")),
                EntryKind::File,
                70,
            ));

            assert_eq!(world.decide().0, StoreVerdict::LeftOut(Reason::OllamaDownload, world.store_sizes()), "{name}");
            assert_eq!(world.store_sizes().logical.bytes(), 4 * MANIFEST + (1..=7).map(listed).sum::<u64>() + 70);
        }
    }

    #[test]
    fn a_foreign_entry_is_told_before_a_partial_download() {
        let partial = in_store(&format!("blobs/{}-partial", digest(9).blob_name()));
        let world = World::mixed()
            .with(matched(OllamaInterest::Blobs, partial.clone(), EntryKind::File, 70))
            .changed(&partial, foreign);

        assert_eq!(world.decide().0, StoreVerdict::LeftOut(Reason::NotYours, world.store_sizes()));
    }

    #[test]
    fn one_unreadable_manifest_leaves_out_the_store() {
        for why in [Unreadable::NotJson, Unreadable::Schema, Unreadable::NotRead(fresher_core::SmallFile::TooLong)] {
            let mut world = World::mixed();
            world.manifests.insert(manifest_path("mistral", "7b"), Err(why.clone()));

            assert_eq!(
                world.decide(),
                (StoreVerdict::LeftOut(Reason::OllamaUnreadable, world.store_sizes()), Vec::new())
            );
        }
    }

    #[test]
    fn manifests_not_all_read_leave_out_the_store() {
        let world = World::mixed();
        let entries = crate::ollama::decide::Entries::new(&world.candidates);

        let verdict = crate::ollama::decide::store(&entries, Vec::new());

        assert_eq!(verdict, StoreVerdict::LeftOut(Reason::OllamaUnreadable, world.store_sizes()));
    }

    #[test]
    fn a_link_in_the_manifests_tree_leaves_out_the_store() {
        let cases = [
            (OllamaInterest::Hosts, "manifests/hf.co"),
            (OllamaInterest::Namespaces, "manifests/registry.ollama.ai/elsewhere"),
            (OllamaInterest::Models, "manifests/registry.ollama.ai/library/linked"),
        ];
        for (which, place) in cases {
            for kind in [EntryKind::Symlink, EntryKind::File, EntryKind::Other] {
                let world = World::mixed().with(matched(which, in_store(place), kind, 30));

                assert_eq!(
                    world.decide(),
                    (StoreVerdict::LeftOut(Reason::OllamaUnreadable, world.store_sizes()), Vec::new()),
                    "{place} {kind:?}"
                );
                let entries = crate::ollama::decide::Entries::new(&world.candidates);
                assert!(crate::ollama::decide::kept(&entries).is_some(), "{place} {kind:?}");
            }
        }
    }

    #[test]
    fn a_hidden_folder_or_link_in_the_manifests_tree_leaves_out_the_store() {
        let cases = [
            (OllamaInterest::Hosts, "manifests/.cache"),
            (OllamaInterest::Namespaces, "manifests/registry.ollama.ai/.hidden"),
            (OllamaInterest::Models, "manifests/registry.ollama.ai/library/.llama3"),
        ];
        for (which, place) in cases {
            for kind in [EntryKind::Folder, EntryKind::Symlink, EntryKind::Other] {
                let world = World::mixed().with(matched(which, in_store(place), kind, 0));

                assert_eq!(
                    world.decide(),
                    (StoreVerdict::LeftOut(Reason::OllamaUnreadable, world.store_sizes()), Vec::new()),
                    "{place} {kind:?}"
                );
            }
        }
    }

    #[test]
    fn a_hidden_file_in_the_manifests_tree_blocks_nothing() {
        let world = World::mixed()
            .with(matched(OllamaInterest::Hosts, in_store("manifests/.DS_Store"), EntryKind::File, 10))
            .with(matched(
                OllamaInterest::Namespaces,
                in_store("manifests/registry.ollama.ai/.DS_Store"),
                EntryKind::File,
                10,
            ))
            .with(matched(
                OllamaInterest::Models,
                in_store("manifests/registry.ollama.ai/library/.x"),
                EntryKind::File,
                0,
            ));

        assert_eq!(world.decide(), World::mixed().decide());
    }

    #[test]
    fn a_missing_layer_leaves_out_the_model() {
        let world = World::mixed().without(&in_store(&format!("blobs/{}", digest(7).blob_name())));

        let (verdict, models) = world.decide();

        assert!(matches!(verdict, StoreVerdict::Open(_)));
        // chat:1, llama3, mistral:7b: chat keeps its manifest and its config; layer 5 is mistral's too.
        assert!(matches!(models[0], ModelVerdict::LeftOut(Reason::OllamaIncomplete, _)));
        assert_eq!(logical(&models[0]), Some(MANIFEST + listed(6)));
        assert!(matches!(models[1], ModelVerdict::Offered(_)));
        assert!(matches!(models[2], ModelVerdict::Offered(_)));
    }

    #[test]
    fn a_layer_of_another_size_leaves_out_the_model() {
        let layer = in_store(&format!("blobs/{}", digest(3).blob_name()));
        let world = World::mixed().changed(&layer, |found| {
            found.sizes.logical = fresher_core::ByteSize::new(listed(3) - 1);
        });

        let models = world.decide().1;

        assert!(matches!(models[1], ModelVerdict::LeftOut(Reason::OllamaIncomplete, _)));
        assert_eq!(logical(&models[1]), Some(2 * MANIFEST + listed(1) + listed(2) + listed(3) - 1));
    }

    #[test]
    fn a_linked_layer_leaves_out_the_model() {
        for kind in [EntryKind::Symlink, EntryKind::Folder, EntryKind::Other] {
            let layer = in_store(&format!("blobs/{}", digest(1).blob_name()));
            let world = World::mixed().changed(&layer, |found| found.kind = kind);

            let models = world.decide().1;

            assert!(matches!(models[1], ModelVerdict::LeftOut(Reason::OllamaIncomplete, _)), "{kind:?}");
        }
    }

    #[test]
    fn too_many_parts_is_not_claimed() {
        // One manifest, the config and 62 layers: 64 parts; a second tag makes 65.
        let layers: Vec<u8> = (2..u8::try_from(MAX_PARTS).unwrap()).collect();
        let fits = World::empty().with_model("big", "1", 1, &layers).with_layers(&[1]).with_layers(&layers);
        let over = World::empty()
            .with_model("big", "1", 1, &layers)
            .with_model("big", "2", 1, &layers)
            .with_layers(&[1])
            .with_layers(&layers);

        assert!(matches!(fits.decide().1[0], ModelVerdict::Offered(_)));
        assert_eq!(over.decide().1, [ModelVerdict::NotClaimed]);
    }

    #[test]
    fn one_tag_whose_every_layer_is_shared_is_not_claimed() {
        let world =
            World::empty().with_model("base", "1", 1, &[2]).with_model("big", "1", 1, &[2, 3]).with_layers(&[1, 2, 3]);

        let models = world.decide().1;

        assert_eq!(models[0], ModelVerdict::NotClaimed);
        assert!(matches!(models[1], ModelVerdict::Offered(_)));
    }

    #[test]
    fn a_model_with_no_time_is_not_claimed() {
        let mut world = World::empty().with_model("m", "1", 1, &[2]).with_layers(&[1, 2]);
        for found in &mut world.candidates {
            found.modified = None;
            found.accessed = None;
        }

        assert_eq!(world.decide().1, [ModelVerdict::NotClaimed]);
    }

    #[test]
    fn a_blob_or_manifest_off_the_shape_is_neither_counted_nor_claimed() {
        let extra = [
            matched(OllamaInterest::Blobs, in_store("blobs/.DS_Store"), EntryKind::File, 10),
            matched(OllamaInterest::Manifests, in_store("manifests/h/n/.m/t"), EntryKind::File, 10),
            matched(OllamaInterest::Manifests, in_store("manifests/h/n/m/folder"), EntryKind::Folder, 0),
        ];
        let plain = World::mixed();
        let world = extra.into_iter().fold(World::mixed(), World::with).changed(&in_store("blobs/.DS_Store"), foreign);

        assert_eq!(world.decide(), plain.decide());
        assert_eq!(world.store_sizes(), plain.store_sizes());
        assert_eq!(layer(1).kind, EntryKind::File);
    }
}

mod positive {
    use fresher_core::{ByteSize, Timestamp};
    use proptest::prelude::*;

    use super::{ACCESSED, MANIFEST, World};
    use crate::ollama::decide::ModelVerdict;
    use crate::ollama::test_support::{digest, in_store, listed, manifest_path};

    #[test]
    fn a_store_that_can_still_be_offered_is_kept_by_no_row_before_reading() {
        let world = World::mixed();

        assert_eq!(crate::ollama::decide::kept(&crate::ollama::decide::Entries::new(&world.candidates)), None);
    }

    #[test]
    fn a_model_is_offered_with_its_freed_sizes() {
        let models = World::mixed().decide().1;

        let [ModelVerdict::Offered(chat), ModelVerdict::Offered(llama), ModelVerdict::Offered(mistral)] =
            models.as_slice()
        else {
            unreachable!("three offers: {models:?}");
        };
        let blob = |byte: u8| in_store(&format!("blobs/{}", digest(byte).blob_name()));
        assert_eq!(
            llama.parts.all(),
            [manifest_path("llama3", "8b"), manifest_path("llama3", "latest"), blob(1), blob(2), blob(3)]
        );
        assert_eq!(llama.freed.logical.bytes(), 2 * MANIFEST + listed(1) + listed(2) + listed(3));
        assert_eq!(llama.freed.allocated.bytes(), 2 * 4096 + 3 * 4096);
        assert_eq!(llama.listed, ByteSize::new(listed(1) + listed(2) + listed(3)));
        assert_eq!(llama.sharers, Vec::<String>::new());
        assert_eq!(llama.date, Timestamp::from_secs(ACCESSED));

        assert_eq!(chat.parts.all(), [manifest_path("chat", "1"), blob(6), blob(7)]);
        assert_eq!(chat.freed.logical.bytes(), MANIFEST + listed(6) + listed(7));
        assert_eq!(chat.listed, ByteSize::new(listed(5) + listed(6) + listed(7)));
        assert_eq!(chat.sharers, ["mistral:7b"]);
        assert_eq!(mistral.parts.all(), [manifest_path("mistral", "7b"), blob(4)]);
        assert_eq!(mistral.sharers, ["chat:1"]);
    }

    #[test]
    fn the_date_is_the_manifests_change_when_no_freed_layer_was_opened_since() {
        let mut world = World::empty().with_model("m", "1", 1, &[2]).with_layers(&[1, 2]);
        for found in &mut world.candidates {
            found.accessed = None;
        }

        let models = world.decide().1;

        let [ModelVerdict::Offered(offer)] = models.as_slice() else { unreachable!("one offer: {models:?}") };
        assert_eq!(offer.date, Timestamp::from_secs(super::CHANGED));
    }

    proptest! {
        #[test]
        fn a_restored_scan_decides_the_same(
            shuffled in Just(World::mixed().candidates).prop_shuffle(),
            missing in 0usize..9,
        ) {
            // The same matches in another order, as a restored scan hands them, with one layer gone or none.
            let gone = in_store(&format!("blobs/{}", digest(u8::try_from(missing).unwrap()).blob_name()));
            let walked = World::mixed().without(&gone);
            let restored = World { candidates: shuffled, ..World::mixed() }.without(&gone);

            prop_assert_eq!(restored.decide(), walked.decide());
        }
    }
}
