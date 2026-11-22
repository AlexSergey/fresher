use std::collections::HashSet;
use std::time::{Duration, SystemTime};

use fresher_core::CategoryId;
use serde::{Deserialize, Serialize};

use crate::CaseError;
use crate::case::{category_id, check_category, leak};

/// The `[demo]` header of a demo world (`fixtures/demos/<world>.toml`): the world's own clock, how its sparse files
/// take space, the categories it must offer and the volume it is on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Demo {
    now: SystemTime,
    sizes: DemoSizes,
    shows: Vec<CategoryId>,
    volume: Option<DemoVolume>,
}

/// How the world's sparse files take space on disk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DemoSizes {
    /// As on disk: a sparse file takes none.
    #[default]
    Sparse,
    /// Every sparse file takes its declared size, stated.
    AsDeclared,
}

/// The volume a demo world states it is on, in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoVolume {
    name: String,
    capacity: u64,
    used: u64,
    free: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct RawDemo {
    now: u64,
    #[serde(default)]
    sizes: DemoSizes,
    shows: Vec<String>,
    volume: Option<RawVolume>,
}

/// A `[demo.volume]`, and the `[volume]` of `world.toml`.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub(crate) struct RawVolume {
    name: String,
    capacity: u64,
    used: u64,
    free: u64,
}

impl Demo {
    /// Checks the header of the case file `feature`; only a header that passes every check keeps its ids.
    pub(crate) fn from_raw(feature: &str, raw: RawDemo) -> Result<Self, CaseError> {
        let refused = |reason: String| CaseError::Demo { feature: feature.into(), reason };
        if raw.now == 0 {
            return Err(refused("`now` must be a moment after the Unix epoch, in seconds".into()));
        }
        let now = SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(raw.now))
            .ok_or_else(|| refused(format!("`now` {} is past what the clock can hold", raw.now)))?;
        if raw.shows.is_empty() {
            return Err(refused("`shows` must name at least one category".into()));
        }
        let mut seen = HashSet::new();
        for id in &raw.shows {
            check_category(id)
                .map_err(|source| refused(format!("`shows` holds `{id}`, not a category id: {source}")))?;
            if !seen.insert(id.as_str()) {
                return Err(refused(format!("`shows` names `{id}` twice")));
            }
        }
        let volume = raw.volume.map(|volume| DemoVolume::from_raw(volume).map_err(refused)).transpose()?;
        let shows = raw
            .shows
            .into_iter()
            .map(|id| {
                let kept = leak(id.clone());
                category_id(kept)
                    .map_err(|source| refused(format!("`shows` holds `{id}`, not a category id: {source}")))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { now, sizes: raw.sizes, shows, volume })
    }

    /// The world's clock: the moment it is generated at, never the wall clock.
    pub const fn now(&self) -> SystemTime {
        self.now
    }

    /// How the world's sparse files take space.
    pub const fn sizes(&self) -> DemoSizes {
        self.sizes
    }

    /// The categories the world must offer, in the file's order.
    pub fn shows(&self) -> &[CategoryId] {
        &self.shows
    }

    /// The volume the world states it is on; `None` leaves the Mac's.
    pub const fn volume(&self) -> Option<&DemoVolume> {
        self.volume.as_ref()
    }
}

impl DemoVolume {
    /// Refuses an empty name, no capacity, or more used and free than the capacity, naming the field.
    pub(crate) fn from_raw(raw: RawVolume) -> Result<Self, String> {
        if raw.name.is_empty() {
            return Err("`volume.name` must not be empty".into());
        }
        if raw.capacity == 0 {
            return Err("`volume.capacity` must be more than 0".into());
        }
        if raw.used.checked_add(raw.free).is_none_or(|taken| taken > raw.capacity) {
            return Err("`volume.used` and `volume.free` together must not exceed `volume.capacity`".into());
        }
        Ok(Self { name: raw.name, capacity: raw.capacity, used: raw.used, free: raw.free })
    }

    /// The volume as `world.toml` writes it.
    pub(crate) fn to_raw(&self) -> RawVolume {
        RawVolume { name: self.name.clone(), capacity: self.capacity, used: self.used, free: self.free }
    }

    /// The volume's name, as the Finder shows it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Its capacity in bytes.
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    /// The bytes in use.
    pub const fn used(&self) -> u64 {
        self.used
    }

    /// The bytes free.
    pub const fn free(&self) -> u64 {
        self.free
    }
}

#[cfg(test)]
mod tests {
    const HEADER: &str =
        "feature = \"developer-mac\"\n[demo]\nnow = 1795000000\nshows = [\"dev-caches.xcode-derived-data\"]\n";

    const SCENERY: &str = r#"
[[case]]
name = "Xcode's build products"
kind = "scenery"
[[case.file]]
path = "home/Library/Developer/Xcode/DerivedData/Atlas-bxkqzr/Build/Atlas"
size = 6442450944
age = "40d"
"#;

    const MORE_SCENERY: &str = r#"
[[case]]
name = "a browser's cache"
kind = "scenery"
[[case.file]]
path = "home/Library/Caches/com.example.Browser/data.bin"
size = 1048576
"#;

    /// A demo file: `header` (the `feature` and `[demo]` lines and any table of the header), then `cases`.
    fn world(header: &str, cases: &[&str]) -> String {
        format!("{header}{}", cases.concat())
    }

    mod negative {
        use super::{HEADER, SCENERY, world};
        use crate::{CaseError, CaseFile};

        fn refused(text: &str) -> CaseError {
            CaseFile::parse(text).unwrap_err()
        }

        /// The message of the `[demo]` refusal of `header`, which names the file and the field.
        fn demo_reason(header: &str) -> String {
            let error = refused(&world(header, &[SCENERY]));
            assert!(matches!(&error, CaseError::Demo { feature, .. } if feature == "developer-mac"), "{error}");
            error.to_string()
        }

        #[test]
        fn a_demo_file_that_names_a_module_is_refused() {
            let text = world(&HEADER.replace("[demo]", "module = \"trial\"\n[demo]"), &[SCENERY]);

            let error = refused(&text);

            assert!(matches!(&error, CaseError::DemoWithModule { feature } if feature == "developer-mac"));
            assert!(error.to_string().contains("developer-mac"));
        }

        #[test]
        fn a_negative_or_positive_case_in_a_demo_file_is_refused() {
            for kind in ["negative", "positive"] {
                let text = world(HEADER, &[&SCENERY.replace("\"scenery\"", &format!("\"{kind}\""))]);

                let error = refused(&text);

                assert!(
                    matches!(&error, CaseError::CheckedCaseInDemo { case } if case == "Xcode's build products"),
                    "{kind}: {error}"
                );
                assert!(error.to_string().contains("Xcode's build products"));
            }
        }

        #[test]
        fn a_scenery_case_without_a_demo_header_is_refused() {
            let text = format!("feature = \"trial\"\n{SCENERY}");

            let error = refused(&text);

            assert!(matches!(&error, CaseError::SceneryOutsideDemo { case } if case == "Xcode's build products"));
            assert!(error.to_string().contains("Xcode's build products"));
        }

        #[test]
        fn a_scenery_case_with_an_expect_a_map_a_left_out_or_an_insight_is_refused() {
            let expect = "[case.expect]\nitem = \"home/Library\"\ncategory = \"trial.caches\"\ndanger = \"low\"\n\
                          after-cleanup = \"in-trash\"\n";
            let map = "[case.map]\npart = \"explained\"\n";
            let left_out =
                "[[case.left-out]]\ncategory = \"trial.caches\"\nreason = \"trial.error.failed\"\nitems = 1\n";
            let insight = "[[case.insight]]\nidentity = \"orbstack\"\nfigures = 1\nparts = 0\ncommands = 0\n";

            for extra in [expect, map, left_out, insight] {
                let error = refused(&world(HEADER, &[&format!("{SCENERY}{extra}")]));

                assert!(
                    matches!(&error, CaseError::SceneryWithExpectation { case } if case == "Xcode's build products"),
                    "{extra}: {error}"
                );
            }
        }

        #[test]
        fn a_now_of_zero_is_refused() {
            assert!(demo_reason(&HEADER.replace("1795000000", "0")).contains("`now`"));
        }

        #[test]
        fn a_now_the_clock_cannot_hold_is_refused() {
            assert!(demo_reason(&HEADER.replace("1795000000", &u64::MAX.to_string())).contains("`now`"));
        }

        #[test]
        fn an_empty_shows_is_refused() {
            assert!(demo_reason(&HEADER.replace("[\"dev-caches.xcode-derived-data\"]", "[]")).contains("`shows`"));
        }

        #[test]
        fn a_shows_entry_that_is_no_category_id_is_refused() {
            for id in ["xcode-derived-data", "Dev-caches.derived", "dev-caches.", "dev-caches.Derived"] {
                let header = HEADER.replace("dev-caches.xcode-derived-data", id);

                let reason = demo_reason(&header);

                assert!(reason.contains("`shows`") && reason.contains(id), "{id}: {reason}");
            }
        }

        #[test]
        fn a_category_shown_twice_is_refused() {
            let header = HEADER.replace(
                "[\"dev-caches.xcode-derived-data\"]",
                "[\"dev-caches.xcode-derived-data\", \"dev-caches.xcode-derived-data\"]",
            );

            assert!(demo_reason(&header).contains("twice"));
        }

        #[test]
        fn an_unknown_key_or_a_missing_one_in_the_header_is_refused() {
            for header in [
                format!("{HEADER}colour = \"blue\"\n"),
                format!("{HEADER}sizes = \"huge\"\n"),
                HEADER.replace("now = 1795000000\n", ""),
                HEADER.replace("shows = [\"dev-caches.xcode-derived-data\"]\n", ""),
                format!("{HEADER}[demo.volume]\nname = \"Mac\"\ncapacity = 10\nused = 1\nfree = 1\nkind = \"ssd\"\n"),
            ] {
                assert!(matches!(refused(&world(&header, &[SCENERY])), CaseError::Parse { .. }), "{header}");
            }
        }

        #[test]
        fn a_volume_with_more_used_and_free_than_its_capacity_is_refused() {
            for (used, free) in [(6, 5), (u64::MAX, 1)] {
                let header =
                    format!("{HEADER}[demo.volume]\nname = \"Mac\"\ncapacity = 10\nused = {used}\nfree = {free}\n");

                assert!(demo_reason(&header).contains("`volume.capacity`"), "{used} + {free}");
            }
        }

        #[test]
        fn a_volume_with_an_empty_name_or_no_capacity_is_refused() {
            let empty = format!("{HEADER}[demo.volume]\nname = \"\"\ncapacity = 10\nused = 1\nfree = 1\n");
            let none = format!("{HEADER}[demo.volume]\nname = \"Mac\"\ncapacity = 0\nused = 0\nfree = 0\n");

            assert!(demo_reason(&empty).contains("`volume.name`"));
            assert!(demo_reason(&none).contains("`volume.capacity`"));
        }
    }

    mod positive {
        use std::time::{Duration, SystemTime};

        use super::{HEADER, MORE_SCENERY, SCENERY, world};
        use crate::{CaseFile, CaseKind, DemoSizes};

        #[test]
        fn a_demo_file_with_two_scenery_cases_parses_with_its_header() {
            let header = format!(
                "{}sizes = \"as-declared\"\n[demo.volume]\nname = \"Macintosh HD\"\ncapacity = 494384795648\n\
                 used = 401234567890\nfree = 93150227758\n",
                HEADER.replace(
                    "[\"dev-caches.xcode-derived-data\"]",
                    "[\"dev-caches.xcode-derived-data\", \"app-caches.library-caches\"]"
                )
            );

            let file = CaseFile::parse(&world(&header, &[SCENERY, MORE_SCENERY])).unwrap();

            let demo = file.demo().unwrap();
            assert_eq!(demo.now(), SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000));
            assert_eq!(demo.sizes(), DemoSizes::AsDeclared);
            let shows: Vec<_> = demo.shows().iter().map(|id| id.as_str()).collect();
            assert_eq!(shows, ["dev-caches.xcode-derived-data", "app-caches.library-caches"]);
            let volume = demo.volume().unwrap();
            assert_eq!(
                (volume.name(), volume.capacity(), volume.used(), volume.free()),
                ("Macintosh HD", 494_384_795_648, 401_234_567_890, 93_150_227_758)
            );
            assert_eq!(file.module(), None);
            let kinds: Vec<_> = file.cases().iter().map(|case| (case.name(), case.kind())).collect();
            assert_eq!(
                kinds,
                [("Xcode's build products", CaseKind::Scenery), ("a browser's cache", CaseKind::Scenery)]
            );
        }

        #[test]
        fn a_header_without_sizes_or_volume_is_sparse_on_the_macs_volume() {
            let file = CaseFile::parse(&world(HEADER, &[SCENERY])).unwrap();

            let demo = file.demo().unwrap();
            assert_eq!(demo.sizes(), DemoSizes::Sparse);
            assert_eq!(demo.volume(), None);
        }

        #[test]
        fn a_test_case_file_has_no_demo_header() {
            let text = "feature = \"trial\"\n[[case]]\nname = \"n\"\nkind = \"negative\"\n";

            assert_eq!(CaseFile::parse(text).unwrap().demo(), None);
        }
    }
}
