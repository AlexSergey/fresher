//! The names on plain values: each refusal first, then the forms read.

use std::ffi::OsStr;

use crate::name::{DeviceSupportName, device_support};

/// The device support name of `text`.
fn read(text: &str) -> Option<DeviceSupportName> {
    device_support(OsStr::new(text))
}

mod negative {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;
    use std::path::Path;

    use super::read;
    use crate::name::{archive, device_support};

    #[test]
    fn a_plain_word_is_no_device_support_name() {
        for name in ["Notes", "", "()", " (21C62)"] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn a_version_of_one_number_is_refused() {
        assert_eq!(read("17 (21A1)"), None);
    }

    #[test]
    fn a_build_without_its_space_or_parentheses_is_refused() {
        for name in ["17.2(21C62)", "17.2 21C62", "17.2 (21C62", "17.2 21C62)", "17.2 (21C62) "] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn an_empty_build_is_refused() {
        for name in ["17.2 ()", "17.2 (21-C62)", "17.2 (21 C62)"] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn four_version_numbers_are_refused() {
        assert_eq!(read("1.2.3.4 (A1)"), None);
    }

    #[test]
    fn a_version_that_is_not_plain_numbers_is_refused() {
        for name in ["+17.2 (21C62)", "17.-2 (21C62)", "17..2 (21C62)", "17.2. (21C62)", "17.99999999999 (A1)"] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn a_model_that_is_not_one_word_is_refused() {
        for name in ["iPhone 15 17.2 (21C62)", "iPhone15,2  17.2 (21C62)", "iPhone-15 17.2 (21C62)", " 17.2 (21C62)"] {
            assert_eq!(read(name), None, "{name}");
        }
    }

    #[test]
    fn a_name_not_in_utf8_is_refused() {
        assert_eq!(device_support(OsStr::from_bytes(b"17.2 (21C\xff62)")), None);
        assert_eq!(archive(Path::new(OsStr::from_bytes(b"/Archives/2026-03-01/Sh\xffop.xcarchive"))), None);
    }

    #[test]
    fn an_archive_outside_a_dated_folder_is_refused() {
        for path in [
            "Archives/x/A.xcarchive",
            "Archives/2026-13-01/A.xcarchive",
            "Archives/2026-1-01/A.xcarchive",
            "Archives/2026-00-01/A.xcarchive",
            "Archives/2026-01-32/A.xcarchive",
            "Archives/2026-01-00/A.xcarchive",
            "Archives/2026/01/A.xcarchive",
            "Archives/２０２６-01-01/A.xcarchive",
            "A.xcarchive",
        ] {
            assert_eq!(archive(Path::new(path)), None, "{path}");
        }
    }

    #[test]
    fn a_bare_suffix_is_no_archive() {
        for path in ["Archives/2026-03-01/.xcarchive", "Archives/2026-03-01/Shop", "Archives/2026-03-01/xcarchive"] {
            assert_eq!(archive(Path::new(path)), None, "{path}");
        }
    }
}

mod positive {
    use std::path::Path;

    use super::read;
    use crate::name::{ArchiveName, archive};

    #[test]
    fn trailing_zeros_do_not_make_another_version() {
        let version = |text: &str| read(&format!("{text} (21C62)")).unwrap().version;

        assert_eq!(version("17.2"), version("17.2.0"));
        assert_eq!(version("17.02"), version("17.2.0"));
        assert!(version("17.2.1") > version("17.2.0"));
        assert!(version("17.10") > version("17.9.0"));
        assert_eq!(version("17.2.0").to_string(), "17.2.0");
    }

    #[test]
    fn a_version_and_build_is_read() {
        let name = read("17.2 (21C62)").unwrap();

        assert_eq!(name.model, None);
        assert_eq!(name.version.to_string(), "17.2");
        assert_eq!(name.build, "21C62");
    }

    #[test]
    fn a_model_prefix_is_read() {
        let phone = read("iPhone15,2 17.2 (21C62)").unwrap();
        let watch = read("Watch6,1 10.0 (21R355)").unwrap();

        assert_eq!(
            (phone.model.as_deref(), phone.version.to_string(), phone.build.as_str()),
            (Some("iPhone15,2"), "17.2".into(), "21C62")
        );
        assert_eq!(
            (watch.model.as_deref(), watch.version.to_string(), watch.build.as_str()),
            (Some("Watch6,1"), "10.0".into(), "21R355")
        );
    }

    #[test]
    fn three_numbers_are_read() {
        let name = read("16.4.1 (20E252)").unwrap();

        assert_eq!(name.version.to_string(), "16.4.1");
        assert_eq!(name.build, "20E252");
    }

    #[test]
    fn versions_compare_by_number() {
        let version = |text: &str| read(&format!("{text} (A1)")).unwrap().version;

        assert!(version("17.10") > version("17.9"));
        assert!(version("17.2.1") > version("17.2"));
        assert!(version("18.0") > version("17.10.3"));
        assert_eq!(version("17.02"), version("17.2"));
        assert_eq!(version("17.02").to_string(), "17.2");
    }

    #[test]
    fn an_archive_is_read() {
        let found =
            archive(Path::new("/home/Library/Developer/Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive"));

        assert_eq!(found, Some(ArchiveName { name: "Shop 01-03-2026, 10.15".into(), date: "2026-03-01".into() }));
    }

    #[test]
    fn the_suffix_ignores_case() {
        let found = archive(Path::new("Archives/2026-12-31/Shop.XCArchive"));

        assert_eq!(found, Some(ArchiveName { name: "Shop".into(), date: "2026-12-31".into() }));
    }
}
