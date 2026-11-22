//! Where an app keeps its files outside its bundle: one row per place, each a known folder (or one folder below it)
//! whose first-level children are named after the app's bundle id.

use fresher_core::{FootprintKind, KnownFolder};

/// How a child of a place is named after an app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NameShape {
    /// `<id>`.
    Id,
    /// `<id><suffix>`.
    IdWithSuffix(&'static str),
    /// `<id><suffix>` or `<id>.<more><suffix>`.
    IdDotted(&'static str),
    /// `<team id>.<anything>`.
    TeamPrefixed,
}

/// One place of an app's footprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) kind: FootprintKind,
    pub(crate) folder: KnownFolder,
    /// The folders below `folder`, in order, that hold the children.
    pub(crate) under: &'static [&'static str],
    pub(crate) name: NameShape,
}

const fn place(kind: FootprintKind, folder: KnownFolder, under: &'static [&'static str], name: NameShape) -> Place {
    Place { kind, folder, under, name }
}

/// Every place of an app's footprint but its bundle, in the order an app's page lists them.
pub(crate) const PLACES: &[Place] = &[
    place(FootprintKind::Caches, KnownFolder::AppCaches, &[], NameShape::Id),
    place(FootprintKind::Logs, KnownFolder::Logs, &[], NameShape::Id),
    place(FootprintKind::SavedState, KnownFolder::SavedState, &[], NameShape::IdWithSuffix(".savedState")),
    place(FootprintKind::LaunchAgents, KnownFolder::Library, &["LaunchAgents"], NameShape::IdDotted(".plist")),
    place(FootprintKind::WebData, KnownFolder::Library, &["HTTPStorages"], NameShape::Id),
    place(FootprintKind::WebData, KnownFolder::Library, &["HTTPStorages"], NameShape::IdWithSuffix(".binarycookies")),
    place(FootprintKind::WebData, KnownFolder::Library, &["WebKit"], NameShape::Id),
    place(FootprintKind::WebData, KnownFolder::Library, &["Cookies"], NameShape::IdWithSuffix(".binarycookies")),
    place(FootprintKind::Settings, KnownFolder::Preferences, &[], NameShape::IdWithSuffix(".plist")),
    place(FootprintKind::Support, KnownFolder::AppSupport, &[], NameShape::Id),
    place(FootprintKind::Container, KnownFolder::Containers, &[], NameShape::Id),
    place(FootprintKind::GroupContainer, KnownFolder::GroupContainers, &[], NameShape::TeamPrefixed),
];

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::FootprintKind;

        use super::super::PLACES;

        #[test]
        fn the_bundle_has_no_place_of_its_own() {
            assert!(PLACES.iter().all(|place| place.kind != FootprintKind::Bundle));
        }
    }

    mod positive {
        use fresher_core::FootprintKind;

        use super::super::PLACES;

        #[test]
        fn every_kind_but_the_bundle_has_a_place_in_the_kinds_order() {
            let mut kinds: Vec<FootprintKind> = PLACES.iter().map(|place| place.kind).collect();
            kinds.dedup();

            let expected: Vec<FootprintKind> =
                FootprintKind::ALL.into_iter().filter(|&kind| kind != FootprintKind::Bundle).collect();
            assert_eq!(kinds, expected);
        }
    }
}
