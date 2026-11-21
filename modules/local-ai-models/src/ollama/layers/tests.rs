//! The layout of hand-made manifests: aliases, freed layers, sharers, and its properties over generated stores.

use std::path::PathBuf;

use crate::name::OllamaTag;
use crate::ollama::manifest::Manifest;
use crate::ollama::test_support::{manifest, manifest_path, tag};

/// The manifest of `model:tag` made of `config` and `layers`, as `layout` takes it.
fn entry(model: &str, name: &str, config: u8, layers: &[u8]) -> (OllamaTag, PathBuf, Manifest) {
    (tag(model, name), manifest_path(model, name), manifest(config, layers))
}

mod negative {
    use super::entry;
    use crate::ollama::layers::layout;
    use crate::ollama::test_support::digest;

    #[test]
    fn a_layer_another_model_names_is_never_a_part() {
        let layout = layout(vec![entry("base", "1", 1, &[2, 3]), entry("chat", "1", 4, &[2, 3, 5])]);

        assert_eq!(layout.parts(0), [digest(1)]);
        assert_eq!(layout.parts(1), [digest(4), digest(5)]);
    }

    #[test]
    fn an_incomplete_models_layers_still_count_as_named() {
        // `broken` names a layer no blob holds; it still keeps `whole` from freeing the layer they share.
        let layout = layout(vec![entry("broken", "1", 1, &[2, 9]), entry("whole", "1", 3, &[2])]);

        assert_eq!(layout.parts(1), [digest(3)]);
        assert_eq!(layout.sharers(1), [0]);
    }

    #[test]
    fn a_model_out_of_range_has_no_part_and_no_sharer() {
        let layout = layout(vec![entry("one", "1", 1, &[2])]);

        assert_eq!(layout.parts(1), []);
        assert_eq!(layout.sharers(1), Vec::<usize>::new());
    }
}

mod positive {
    use std::collections::BTreeSet;

    use proptest::prelude::*;

    use super::entry;
    use crate::ollama::layers::layout;
    use crate::ollama::test_support::{digest, manifest_path as path, tag};

    #[test]
    fn tags_with_the_same_files_are_one_model() {
        let layout = layout(vec![entry("llama3", "latest", 1, &[2, 3]), entry("llama3", "8b", 1, &[3, 2])]);

        assert_eq!(layout.models().len(), 1);
        let model = &layout.models()[0];
        assert_eq!(model.tags, [tag("llama3", "8b"), tag("llama3", "latest")]);
        assert_eq!(model.manifests, [path("llama3", "8b"), path("llama3", "latest")]);
        assert_eq!(layout.sharers(0), Vec::<usize>::new());
    }

    #[test]
    fn a_model_frees_its_manifests_and_its_unshared_layers() {
        let layout = layout(vec![entry("mistral", "7b", 1, &[2, 3])]);

        assert_eq!(layout.parts(0), [digest(1), digest(2), digest(3)]);
        assert_eq!(layout.models()[0].digests, BTreeSet::from([digest(1), digest(2), digest(3)]));
        assert_eq!(layout.models()[0].listed, BTreeSet::from([(digest(1), 100), (digest(2), 200), (digest(3), 300)]));
    }

    #[test]
    fn sharers_are_listed_by_name() {
        let layout =
            layout(vec![entry("zeta", "1", 4, &[2]), entry("base", "1", 1, &[2]), entry("alpha", "1", 3, &[2])]);

        let names: Vec<&str> = layout.models().iter().map(|model| model.tags[0].model.as_str()).collect();
        assert_eq!(names, ["alpha", "base", "zeta"]);
        assert_eq!(layout.sharers(1), [0, 2]);
        assert_eq!(layout.sharers(0), [1, 2]);
    }

    /// Up to six manifests, each of one config and up to four layers out of eight digests, with distinct tags.
    fn manifests() -> impl Strategy<Value = Vec<(String, u8, Vec<u8>)>> {
        proptest::collection::vec((1u8..=8, proptest::collection::vec(1u8..=8, 1..=4)), 0..=6).prop_map(|models| {
            models.into_iter().enumerate().map(|(at, (config, layers))| (format!("m{at}"), config, layers)).collect()
        })
    }

    proptest! {
        #[test]
        fn the_parts_of_two_models_never_overlap(models in manifests()) {
            let layout = layout(models.iter().map(|(name, config, layers)| entry(name, "1", *config, layers)).collect());

            for first in 0..layout.models().len() {
                for second in (first + 1)..layout.models().len() {
                    let one: BTreeSet<_> = layout.parts(first).into_iter().collect();
                    prop_assert!(layout.parts(second).iter().all(|digest| !one.contains(digest)));
                }
            }
        }

        #[test]
        fn freed_and_shared_layers_add_up_to_every_named_layer(models in manifests()) {
            let layout = layout(models.iter().map(|(name, config, layers)| entry(name, "1", *config, layers)).collect());

            let named: BTreeSet<_> = layout.models().iter().flat_map(|model| model.digests.iter().copied()).collect();
            let freed: Vec<_> = (0..layout.models().len()).flat_map(|model| layout.parts(model)).collect();
            let shared: BTreeSet<_> = named
                .iter()
                .copied()
                .filter(|digest| layout.models().iter().filter(|model| model.digests.contains(digest)).count() > 1)
                .collect();
            let mut every: BTreeSet<_> = freed.iter().copied().collect();
            prop_assert_eq!(every.len(), freed.len());
            prop_assert!(every.iter().all(|digest| !shared.contains(digest)));
            every.extend(shared);
            prop_assert_eq!(every, named);
        }

        #[test]
        fn the_layout_does_not_depend_on_the_order_of_manifests(
            (models, shuffled) in manifests().prop_flat_map(|models| (Just(models.clone()), Just(models).prop_shuffle())),
        ) {
            let entries = |models: &[(String, u8, Vec<u8>)]| -> Vec<_> {
                models.iter().map(|(name, config, layers)| entry(name, "1", *config, layers)).collect()
            };

            prop_assert_eq!(layout(entries(&shuffled)), layout(entries(&models)));
        }
    }

    #[test]
    fn a_digest_listed_twice_in_one_manifest_is_named_once() {
        let layout = layout(vec![entry("m", "1", 1, &[1, 2, 2])]);

        assert_eq!(layout.parts(0), [digest(1), digest(2)]);
    }
}
