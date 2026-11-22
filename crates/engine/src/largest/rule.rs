//! What an item of the home is: the rule the items query asks of every node below the home.

use fresher_core::{ItemRule, ItemVisit, NodeKind, is_app_bundle, is_package};

/// The items of the home: kept files, packages and collapsed subtrees, never inside the home's `Library` or Trash, a
/// hidden name, an app bundle or a link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HomeItems {
    /// The name of the home's `Library` when it is directly under the home.
    library: Option<String>,
    /// The name of the home's Trash when it is directly under the home.
    trash: Option<String>,
}

impl HomeItems {
    pub(crate) const fn new(library: Option<String>, trash: Option<String>) -> Self {
        Self { library, trash }
    }

    fn is_top_skipped(&self, name: &str, depth: u32) -> bool {
        depth == 1 && [&self.library, &self.trash].into_iter().flatten().any(|skipped| skipped == name)
    }
}

impl ItemRule for HomeItems {
    fn visit(&self, name: &str, kind: NodeKind, depth: u32) -> ItemVisit {
        if self.is_top_skipped(name, depth) || name.starts_with('.') || is_app_bundle(name) {
            return ItemVisit::Skip;
        }
        match kind {
            NodeKind::Link => ItemVisit::Skip,
            NodeKind::File | NodeKind::Collapsed => ItemVisit::Item,
            NodeKind::Folder if is_package(name) => ItemVisit::Item,
            NodeKind::Folder => ItemVisit::Enter,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HomeItems;

    fn rule() -> HomeItems {
        HomeItems::new(Some("Library".into()), Some(".Trash".into()))
    }

    mod negative {
        use fresher_core::{ItemRule, ItemVisit, NodeKind};

        use super::{HomeItems, rule};

        #[test]
        fn the_library_is_skipped_only_at_the_top() {
            assert_eq!(rule().visit("Library", NodeKind::Folder, 1), ItemVisit::Skip);
            assert_eq!(rule().visit("Library", NodeKind::Folder, 2), ItemVisit::Enter);
        }

        #[test]
        fn the_trash_is_skipped() {
            let named = HomeItems::new(None, Some("Bin".into()));

            assert_eq!(rule().visit(".Trash", NodeKind::Folder, 1), ItemVisit::Skip);
            assert_eq!(named.visit("Bin", NodeKind::Folder, 1), ItemVisit::Skip);
        }

        #[test]
        fn a_hidden_name_is_skipped() {
            assert_eq!(rule().visit(".gradle", NodeKind::Folder, 1), ItemVisit::Skip);
            assert_eq!(rule().visit(".big.bin", NodeKind::File, 3), ItemVisit::Skip);
        }

        #[test]
        fn an_app_bundle_is_skipped() {
            assert_eq!(rule().visit("Tool.app", NodeKind::Folder, 2), ItemVisit::Skip);
            assert_eq!(rule().visit("Tool.APP", NodeKind::Folder, 1), ItemVisit::Skip);
        }

        #[test]
        fn a_link_is_skipped() {
            assert_eq!(rule().visit("movie.mov", NodeKind::Link, 1), ItemVisit::Skip);
        }
    }

    mod positive {
        use fresher_core::{ItemRule, ItemVisit, NodeKind};

        use super::rule;

        #[test]
        fn a_package_is_one_item() {
            assert_eq!(rule().visit("Trip.photoslibrary", NodeKind::Folder, 2), ItemVisit::Item);
        }

        #[test]
        fn a_plain_folder_is_entered() {
            assert_eq!(rule().visit("Movies", NodeKind::Folder, 1), ItemVisit::Enter);
        }

        #[test]
        fn a_file_is_an_item() {
            assert_eq!(rule().visit("movie.mov", NodeKind::File, 1), ItemVisit::Item);
        }

        #[test]
        fn a_collapsed_subtree_is_an_item() {
            assert_eq!(rule().visit("folded", NodeKind::Collapsed, 4), ItemVisit::Item);
        }
    }
}
