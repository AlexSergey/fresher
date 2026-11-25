//! The `stale-projects` module: what developers' projects leave behind once nobody works in them.
//!
//! It offers the installed packages (`node_modules`), the build output (`target`, `build`, `.gradle`, `.build`,
//! `.dart_tool`) and the Python environments (`.venv`, `venv`) of projects nothing in which changed for a long time,
//! and Rust's build cache, which only makes the next build faster. It never offers source, the project folder, `.git`
//! or framework output (`dist`, `.next` and the like), never runs a tool (not git, cargo or npm), never claims a
//! folder whole, and never looks at a project in `~/Library`, the apps folders, an app bundle, a hidden folder of the
//! home or the home itself.

#![forbid(unsafe_code)]

mod categories;
mod decide;
mod gathered;
mod place;
mod report;
mod table;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use fresher_core::{
    Artifact, CategoryDescriptor, CategoryIcon, CategoryId, Claim, DaySetting, DescriptorError, FindingSink, IdError,
    Interest, InterestError, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup,
    ModuleId, Pattern, Project, ProjectKind, SettingError, Sizes, TextKeyError, Timestamp,
};

use crate::categories::Category;
use crate::decide::{Scan, Verdict};
use crate::gathered::Gathered;
use crate::place::{Group, Places};
use crate::report::{Categories, Item, Report, Where};
pub use crate::table::ArtifactsError;
use crate::table::{Artifacts, Row};

/// `stale-projects.unused-for-days`. Never fewer than a month: a project is not stale the week after you left it.
const UNUSED_FOR: DaySetting = DaySetting::new("unused-for-days", 30, 3650, 365);

/// The days without a build after which Rust's build cache is offered: the user's decision of 2026-11-19, not a
/// setting.
const CACHE_UNUSED_DAYS: u32 = 7;

/// The artifact of a Rust project that holds its caches.
const TARGET: &str = "target";

/// Why the `stale-projects` module could not be built (a bug in its own constants or table).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum StaleProjectsError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Interest(#[from] InterestError),
    #[error(transparent)]
    Setting(#[from] SettingError),
    #[error(transparent)]
    Artifacts(#[from] ArtifactsError),
}

/// The `stale-projects` module.
#[derive(Debug)]
pub struct StaleProjectsModule {
    descriptor: ModuleDescriptor,
    interests: Vec<Interest>,
    artifacts: Artifacts,
    report: Report,
    /// The folders the module matched in the scan, waiting for its `finish`.
    gathered: Gathered,
}

impl StaleProjectsModule {
    /// The module with its descriptor, its table and its interests.
    ///
    /// # Errors
    ///
    /// A [`StaleProjectsError`] when one of its own constants or its table is invalid: a bug, which its tests catch.
    pub fn new() -> Result<Self, StaleProjectsError> {
        let id = ModuleId::new("stale-projects")?;
        let categories = Categories::new(|category| {
            Ok(CategoryDescriptor::new(CategoryId::new(id, category.id())?, category.danger())
                .with_icon(CategoryIcon::Project))
        })?;
        let report = Report::new(categories)?;
        let (categories, problems) = (report.categories(), report.problems());
        let locales = Locales { en: include_str!("../locales/en.json") };
        let settings = vec![UNUSED_FOR.descriptor(id)?];
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Development, categories, settings, locales)?
            .with_problems(problems)?;
        // A bad table fails here, when the module is built.
        let artifacts = Artifacts::shipped()?;
        Ok(Self { descriptor, interests: interests()?, artifacts, report, gathered: Gathered::default() })
    }

    /// Decides and reports one group: every artifact of every member once, by path; then, for every Rust member
    /// whose `target` is not offered, its caches, and that `target` when it is left out, less the offered caches.
    /// An artifact or a cache that is a project's root or holds one is not claimed: a project folder is never offered.
    fn decide_group(
        &self,
        group: &Group<'_>,
        roots: &BTreeSet<&Path>,
        matched: &BTreeSet<PathBuf>,
        scan: &Scan,
        home: &Path,
        out: &mut dyn FindingSink,
    ) {
        let place = Where { home, outermost: &group.outermost.root };
        let staleness = decide::staleness(decide::group_activity(group.members.iter().map(|m| m.last_activity)), scan);
        let mut artifacts: BTreeMap<&Path, (&Artifact, Option<&Row>)> = BTreeMap::new();
        for member in &group.members {
            for artifact in &member.artifacts {
                artifacts.entry(&artifact.path).or_insert_with(|| (artifact, self.row(member, artifact)));
            }
        }
        let mut targets: BTreeMap<&Path, (Verdict, Item<'_>)> = BTreeMap::new();
        for (path, (artifact, row)) in &artifacts {
            // An artifact with no row is not the module's to judge (a rule that cannot tell does not claim).
            let Some(row) = row else { continue };
            if holds_a_project(path, roots) {
                continue;
            }
            let verdict = decide::artifact(artifact, matched.contains(*path), staleness, scan);
            let item = Item {
                category: row.category,
                path,
                sizes: artifact.sizes,
                command: &row.command,
                group: Some(staleness),
            };
            if row.kind == ProjectKind::Rust && row.folder == TARGET {
                targets.insert(path, (verdict, item));
                if matches!(verdict, Verdict::LeftOut(_)) {
                    // Held back until its caches are decided, to count only what is not offered.
                    continue;
                }
            }
            self.report.tell(&item, place, verdict, out);
        }
        let Some(command) = self.artifacts.row(ProjectKind::Rust, TARGET).map(|row| row.command.as_str()) else {
            return;
        };
        let mut seen: BTreeSet<&Path> = BTreeSet::new();
        for member in group.members.iter().filter(|member| member.kinds.contains(&ProjectKind::Rust)) {
            let target = member.root.join(TARGET);
            let held = targets.remove(target.as_path());
            if held.is_some_and(|(verdict, _)| verdict == Verdict::Offered) {
                // The offered `target` holds its caches.
                continue;
            }
            let mut offered: Vec<Sizes> = Vec::new();
            for cache in member.caches.iter().filter(|cache| {
                is_inside(&cache.path, &target) && !holds_a_project(&cache.path, roots) && seen.insert(&cache.path)
            }) {
                let verdict = decide::cache(cache, matched.contains(&cache.path), scan);
                if verdict == Verdict::Offered {
                    offered.push(cache.sizes);
                }
                let item = Item {
                    category: Category::RustBuildCache,
                    path: &cache.path,
                    sizes: cache.sizes,
                    command,
                    group: None,
                };
                self.report.tell(&item, place, verdict, out);
            }
            if let Some((verdict @ Verdict::LeftOut(_), item)) = held {
                let item = Item { sizes: decide::less(item.sizes, &offered), ..item };
                self.report.tell(&item, place, verdict, out);
            }
        }
    }

    /// The row of `artifact` for the first of `member`'s kinds, in `ProjectKind::ALL` order, that lists its folder
    /// name; `None` when it is not right inside the member's root or no kind lists it.
    fn row(&self, member: &Project, artifact: &Artifact) -> Option<&Row> {
        if artifact.path.parent() != Some(member.root.as_path()) {
            return None;
        }
        let name = artifact.path.file_name().and_then(OsStr::to_str)?;
        ProjectKind::ALL
            .into_iter()
            .filter(|kind| member.kinds.contains(kind))
            .find_map(|kind| self.artifacts.row(kind, name))
    }
}

/// `path` is below `folder`, not `folder` itself.
fn is_inside(path: &Path, folder: &Path) -> bool {
    path != folder && path.starts_with(folder)
}

/// `folder` is the root of one of `roots` or holds one.
fn holds_a_project(folder: &Path, roots: &BTreeSet<&Path>) -> bool {
    // By components: a folder's descendants come right after it.
    roots.range(folder..).next().is_some_and(|root| root.starts_with(folder))
}

/// One interest per folder name the finder looks for: every artifact of every kind and the last segment of every
/// cache pattern. Each is an item: a folder named so is never claimed whole.
fn interests() -> Result<Vec<Interest>, InterestError> {
    let artifacts = ProjectKind::ALL.into_iter().flat_map(ProjectKind::artifacts).copied();
    let caches =
        ProjectKind::ALL.into_iter().flat_map(ProjectKind::caches).filter_map(|pattern| pattern.rsplit('/').next());
    let names: BTreeSet<&str> = artifacts.chain(caches).collect();
    names.into_iter().map(|name| Interest::new(Pattern::FolderName(name.to_owned()), Claim::Item, Vec::new())).collect()
}

impl Module for StaleProjectsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    /// Keeps a matched folder for the scan's `finish`; anything else is not claimed.
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
        self.gathered.add(cx.cancel(), matched);
        Ok(())
    }

    /// Decides the projects of the scan, group by group in their roots' order: every artifact and cache offered,
    /// left out with its reason, or not claimed. Nothing is claimed when the home or the projects are not known.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        let matched = self.gathered.take(cx.cancel());
        let Some(places) = Places::of(cx.system()) else { return Ok(()) };
        let Some(projects) = cx.system().projects() else { return Ok(()) };
        let roots: BTreeSet<&Path> = projects.iter().map(|project| project.root.as_path()).collect();
        let scan = Scan {
            now: Timestamp::from_system_time(cx.clock().now()),
            days: UNUSED_FOR.days(cx.settings()),
            restored: cx.is_restored(),
        };
        for group in place::groups(projects, &places) {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            self.decide_group(&group, &roots, &matched, &scan, places.home(), out);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod built {
    mod positive {
        use std::collections::BTreeSet;

        use fresher_core::{Claim, Module as _, Pattern};

        use crate::StaleProjectsModule;

        fn names(module: &StaleProjectsModule) -> Vec<&str> {
            module
                .interests()
                .iter()
                .filter_map(|interest| match interest.pattern() {
                    Pattern::FolderName(name) => Some(name.as_str()),
                    _ => None,
                })
                .collect()
        }

        #[test]
        fn the_interests_are_every_artifact_and_cache_name_once() {
            let module = StaleProjectsModule::new().unwrap();

            let names = names(&module);

            assert_eq!(
                names,
                [
                    ".build",
                    ".dart_tool",
                    ".gradle",
                    ".venv",
                    "build",
                    "incremental",
                    "llvm-cov-target",
                    "node_modules",
                    "target",
                    "venv",
                ]
            );
            assert_eq!(names.iter().collect::<BTreeSet<_>>().len(), names.len());
        }

        #[test]
        fn no_interest_claims_anything_whole() {
            let module = StaleProjectsModule::new().unwrap();

            assert!(module.interests().iter().all(|interest| interest.claim() == Claim::Item));
        }

        #[test]
        fn no_category_needs_a_capability() {
            let module = StaleProjectsModule::new().unwrap();

            let categories = module.descriptor().categories();

            assert_eq!(categories.len(), 4);
            assert!(categories.iter().all(|category| category.needs().is_empty()));
        }

        #[test]
        fn framework_output_is_never_an_interest_a_row_or_a_core_artifact() {
            const FRAMEWORK: [&str; 5] = ["dist", ".next", ".nuxt", ".turbo", ".parcel-cache"];
            let module = StaleProjectsModule::new().unwrap();
            let table = crate::table::Artifacts::shipped().unwrap();

            for folder in FRAMEWORK {
                assert!(!names(&module).contains(&folder), "{folder} is an interest");
                for kind in fresher_core::ProjectKind::ALL {
                    assert!(!kind.artifacts().contains(&folder), "{folder} is an artifact");
                    assert!(table.row(kind, folder).is_none(), "{folder} is a row");
                    assert!(
                        kind.caches().iter().all(|cache| cache.split('/').all(|segment| segment != folder)),
                        "{folder} is a cache segment"
                    );
                }
            }
        }
    }
}
