//! Manifests and blob names on plain values: what is refused first, then what is read.

/// 64 lowercase hex digits made of `digit`.
fn hex(digit: char) -> String {
    digit.to_string().repeat(64)
}

/// A manifest with the config `a` and one layer per digit of `layers`, each of size 10 (the config's 5).
fn manifest(layers: &str) -> String {
    let layers: Vec<String> =
        layers.chars().map(|digit| format!(r#"{{"digest":"sha256:{}","size":10}}"#, hex(digit))).collect();
    format!(
        r#"{{"schemaVersion":2,"config":{{"digest":"sha256:{}","size":5}},"layers":[{}]}}"#,
        hex('a'),
        layers.join(",")
    )
}

mod negative {
    use fresher_core::SmallFile;

    use super::{hex, manifest};
    use crate::ollama::manifest::{BlobName, Unreadable, blob_kind, read};

    #[test]
    fn a_missing_link_or_long_file_is_not_read() {
        for file in
            [SmallFile::Missing, SmallFile::NotAFile, SmallFile::TooLong, SmallFile::NotText, SmallFile::Unreadable]
        {
            assert_eq!(read(file.clone()), Err(Unreadable::NotRead(file)));
        }
    }

    #[test]
    fn text_that_is_not_json_is_unreadable() {
        for text in ["", "llama3", "{\"schemaVersion\":2,", "{] not json"] {
            assert_eq!(read(SmallFile::Text(text.into())), Err(Unreadable::NotJson), "{text}");
        }
    }

    #[test]
    fn another_schema_version_is_unreadable() {
        for version in ["1", "3", "0"] {
            let text = manifest("b").replace("\"schemaVersion\":2", &format!("\"schemaVersion\":{version}"));

            assert_eq!(read(SmallFile::Text(text)), Err(Unreadable::Schema), "{version}");
        }
    }

    #[test]
    fn json_that_is_not_the_schema_is_unreadable() {
        for text in [
            "[]",
            "{}",
            r#"{"schemaVersion":"2","config":{"digest":"x","size":1},"layers":[]}"#,
            r#"{"schemaVersion":2,"layers":[]}"#,
            &manifest("b").replace("\"size\":10", "\"size\":-1"),
            &manifest("b").replace("\"size\":5", "\"size\":\"5\""),
        ] {
            assert_eq!(read(SmallFile::Text(text.into())), Err(Unreadable::Schema), "{text}");
        }
    }

    #[test]
    fn a_digest_off_the_shape_is_unreadable() {
        let good = hex('b');
        for bad in [
            good.to_uppercase(),
            "b".repeat(63),
            "b".repeat(65),
            format!("sha512:{good}"),
            format!("sha256-{good}"),
            format!("../{good}"),
            format!("sha256:../{good}"),
            format!("sha256:{}g", "b".repeat(63)),
            String::new(),
        ] {
            let text = manifest("b").replace(&format!("sha256:{good}"), &bad);
            assert_eq!(read(SmallFile::Text(text.clone())), Err(Unreadable::Digest), "{bad}");
            let text = manifest("b").replace(&format!("sha256:{}", hex('a')), &bad);
            assert_eq!(read(SmallFile::Text(text)), Err(Unreadable::Digest), "config {bad}");
        }
    }

    #[test]
    fn a_manifest_without_layers_is_unreadable() {
        assert_eq!(read(SmallFile::Text(manifest(""))), Err(Unreadable::NoLayers));
    }

    #[test]
    fn a_digest_twice_with_two_sizes_is_unreadable() {
        let text = manifest("bb").replacen("\"size\":10", "\"size\":11", 1);
        assert_eq!(read(SmallFile::Text(text)), Err(Unreadable::SizeMismatch));

        let config_again = manifest("a").replace("\"size\":10", "\"size\":6");
        assert_eq!(read(SmallFile::Text(config_again)), Err(Unreadable::SizeMismatch));
    }

    #[test]
    fn a_blob_name_off_the_shape_is_not_a_layer() {
        let good = hex('c');
        for name in [
            ".DS_Store".to_owned(),
            "sha256-short".to_owned(),
            format!("sha256-{good}.tmp"),
            format!("sha256-{}", good.to_uppercase()),
            format!("sha256:{good}"),
            format!("sha256-{good}-partial-"),
            format!("sha256-{good}-partial-x"),
            format!("sha256-{good}-partial-1-2"),
            format!("sha256-{good}-partially"),
            format!("sha256-{good}x"),
            format!("sha256-é{good}"),
            String::new(),
        ] {
            assert_eq!(blob_kind(name.as_ref()), BlobName::Other, "{name}");
        }
    }

    #[test]
    fn a_blob_name_not_in_utf8_is_not_a_layer() {
        use std::os::unix::ffi::OsStrExt as _;

        assert_eq!(blob_kind(std::ffi::OsStr::from_bytes(b"sha256-\xff")), BlobName::Other);
    }
}

mod positive {
    use fresher_core::SmallFile;

    use super::{hex, manifest};
    use crate::ollama::manifest::{BlobName, Digest, blob_kind, read};

    fn digest(digit: char) -> Digest {
        Digest::parse(&format!("sha256:{}", hex(digit))).expect("a digest")
    }

    #[test]
    fn a_digest_names_its_blob() {
        assert_eq!(digest('b').blob_name(), format!("sha256-{}", hex('b')));
        let mixed = "0123456789abcdef".repeat(4);

        let parsed = Digest::parse(&format!("sha256:{mixed}")).expect("a digest");

        assert_eq!(parsed.blob_name(), format!("sha256-{mixed}"));
    }

    #[test]
    fn reads_the_config_and_the_layers() {
        let read = read(SmallFile::Text(manifest("bc"))).expect("a manifest");

        assert_eq!(read.config, digest('a'));
        assert_eq!(read.layers, [digest('b'), digest('c')]);
        assert_eq!(read.digests(), [digest('a'), digest('b'), digest('c')]);
        assert_eq!(read.sizes.get(&digest('a')), Some(&5));
        assert_eq!(read.sizes.get(&digest('c')), Some(&10));
    }

    #[test]
    fn unknown_fields_are_allowed() {
        let text = manifest("b")
            .replace("\"schemaVersion\":2", "\"schemaVersion\":2,\"mediaType\":\"x\",\"extra\":{\"a\":[1]}")
            .replace("\"size\":10", "\"size\":10,\"mediaType\":\"y\"");

        assert!(read(SmallFile::Text(text)).is_ok());
    }

    #[test]
    fn a_digest_listed_twice_counts_once() {
        let read = read(SmallFile::Text(manifest("bb"))).expect("a manifest");

        assert_eq!(read.layers, [digest('b')]);
        assert_eq!(read.digests(), [digest('a'), digest('b')]);
    }

    #[test]
    fn the_config_listed_as_a_layer_counts_once_in_the_digests() {
        let read = read(SmallFile::Text(manifest("a").replace("\"size\":10", "\"size\":5"))).expect("a manifest");

        assert_eq!(read.digests(), [digest('a')]);
    }

    #[test]
    fn a_manifest_of_4096_bytes_is_read() {
        let mut text = manifest("b");
        text.push_str(&" ".repeat(4096 - text.len()));
        assert_eq!(text.len(), 4096);

        assert!(read(SmallFile::Text(text)).is_ok());
    }

    #[test]
    fn a_plain_name_is_a_layer_and_both_partial_names_are_partial_downloads() {
        let good = hex('d');

        assert_eq!(blob_kind(format!("sha256-{good}").as_ref()), BlobName::Layer(digest('d')));
        assert_eq!(blob_kind(format!("sha256-{good}-partial").as_ref()), BlobName::Partial(digest('d')));
        assert_eq!(blob_kind(format!("sha256-{good}-partial-0").as_ref()), BlobName::Partial(digest('d')));
        assert_eq!(blob_kind(format!("sha256-{good}-partial-12").as_ref()), BlobName::Partial(digest('d')));
    }
}
