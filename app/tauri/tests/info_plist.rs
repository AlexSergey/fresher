//! `Info.plist`, read as text: the five reasons macOS shows in its prompts (the design of plan 006, section 7) and no
//! other usage key, no privileged helper.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support {
    use std::path::PathBuf;

    /// The design's five keys and their texts, word for word.
    pub(crate) const REASONS: [(&str, &str); 5] = [
        (
            "NSDesktopFolderUsageDescription",
            "Fresher measures what takes space on your Desktop. It reads names and sizes only, and removes nothing \
             unless you confirm it.",
        ),
        (
            "NSDocumentsFolderUsageDescription",
            "Fresher measures what takes space in Documents, such as old project folders. It reads names and sizes \
             only, and removes nothing unless you confirm it.",
        ),
        (
            "NSDownloadsFolderUsageDescription",
            "Fresher measures what takes space in Downloads, such as old installers. It reads names and sizes only, \
             and removes nothing unless you confirm it.",
        ),
        (
            "NSRemovableVolumesUsageDescription",
            "You added this drive to the folders Fresher scans. It reads names and sizes only, and removes nothing \
             unless you confirm it.",
        ),
        (
            "NSNetworkVolumesUsageDescription",
            "You added this network folder to the folders Fresher scans. It reads names and sizes only, and removes \
             nothing unless you confirm it.",
        ),
    ];

    pub(crate) fn plist() -> String {
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Info.plist")).unwrap()
    }

    /// Every `<key>` of the plist with the `<string>` that follows it, in order.
    pub(crate) fn entries(plist: &str) -> Vec<(String, String)> {
        let mut entries = Vec::new();
        let mut rest = plist;
        while let Some((_, after)) = rest.split_once("<key>") {
            let (key, after) = after.split_once("</key>").unwrap();
            let (_, after) = after.split_once("<string>").unwrap();
            let (value, after) = after.split_once("</string>").unwrap();
            entries.push((key.trim().to_owned(), value.trim().to_owned()));
            rest = after;
        }
        entries
    }

    /// The keys of the plist that declare a use of something guarded.
    pub(crate) fn usage_keys(plist: &str) -> Vec<String> {
        entries(plist).into_iter().map(|(key, _)| key).filter(|key| key.ends_with("UsageDescription")).collect()
    }
}

mod negative {
    use crate::support::{REASONS, entries, plist, usage_keys};

    #[test]
    fn the_plist_declares_no_other_usage_key() {
        let expected: Vec<&str> = REASONS.iter().map(|(key, _)| *key).collect();
        let with_apple_events =
            plist().replace("</dict>", "<key>NSAppleEventsUsageDescription</key><string>x</string></dict>");

        assert_eq!(usage_keys(&plist()), expected);
        assert!(!plist().contains("NSAppleEventsUsageDescription"));
        assert_eq!(usage_keys(&with_apple_events).last().map(String::as_str), Some("NSAppleEventsUsageDescription"));
        assert_eq!(entries(&plist()).len(), REASONS.len(), "the plist holds nothing but the five reasons");
    }

    #[test]
    fn the_plist_declares_no_privileged_helper() {
        assert!(!plist().contains("SMPrivilegedExecutables"));
        assert!(!plist().contains("SMAuthorizedClients"));
    }

    #[test]
    fn no_reason_promises_to_delete() {
        for (key, text) in entries(&plist()) {
            let lower = text.to_lowercase();
            assert!(!lower.contains("will delete") && !lower.contains("will remove"), "{key}: {text}");
        }
    }
}

mod positive {
    use crate::support::{REASONS, entries, plist};

    #[test]
    fn the_plist_holds_the_five_reason_strings() {
        let expected: Vec<(String, String)> =
            REASONS.iter().map(|(key, text)| ((*key).to_owned(), (*text).to_owned())).collect();

        let found = entries(&plist());

        assert_eq!(found, expected);
        assert!(found.iter().all(|(_, text)| !text.is_empty()));
    }

    #[test]
    fn every_reason_says_nothing_is_removed_without_a_confirmation() {
        for (key, text) in entries(&plist()) {
            assert!(text.to_lowercase().contains("removes nothing unless you confirm it"), "{key}: {text}");
        }
    }
}
