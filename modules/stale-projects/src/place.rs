//! Which projects are projects here, and their groups (design, section 3.2): never the home or a folder holding it,
//! nothing inside `~/Library`, an apps folder, an app bundle or a hidden folder of the home. Paths are compared by
//! components, never as text.

use std::path::{Component, Path, PathBuf};

use fresher_core::{KnownFolder, Project, SystemFacts};

/// What an app bundle's name ends with, ASCII case ignored.
const APP_SUFFIX: &[u8] = b".app";

/// The places that decide whether a project is one here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Places {
    home: PathBuf,
    /// The located `Library` and apps folders: what lies inside them is never a project here.
    outside: Vec<PathBuf>,
}

impl Places {
    /// The places of `system`; `None` when the home cannot be located, and then nothing is a project here.
    pub(crate) fn of(system: &SystemFacts) -> Option<Self> {
        let home = system.locate(KnownFolder::Home)?;
        let outside = [
            KnownFolder::Library,
            KnownFolder::Applications,
            KnownFolder::UserApplications,
            KnownFolder::SystemApplications,
        ]
        .into_iter()
        .filter_map(|folder| system.locate(folder))
        .collect();
        Some(Self { home, outside })
    }

    /// The home folder, for the findings' identities.
    pub(crate) fn home(&self) -> &Path {
        &self.home
    }

    /// Whether a project at `root` is one here: not the home or a folder holding it, not inside `Library` or an apps
    /// folder, not inside a folder whose name ends `.app` (ASCII case ignored), not inside a hidden folder of the home.
    pub(crate) fn holds(&self, root: &Path) -> bool {
        if self.home.starts_with(root) || self.outside.iter().any(|outside| root.starts_with(outside)) {
            return false;
        }
        if root
            .components()
            .any(|component| matches!(component, Component::Normal(name) if is_app(name.as_encoded_bytes())))
        {
            return false;
        }
        let hidden =
            root.strip_prefix(&self.home).ok().and_then(|below| below.components().next()).is_some_and(
                |first| matches!(first, Component::Normal(name) if name.as_encoded_bytes().starts_with(b".")),
            );
        !hidden
    }
}

/// A name ending `.app`, ASCII case ignored.
fn is_app(name: &[u8]) -> bool {
    name.len()
        .checked_sub(APP_SUFFIX.len())
        .and_then(|cut| name.get(cut..))
        .is_some_and(|end| end.eq_ignore_ascii_case(APP_SUFFIX))
}

/// A project held here and the held projects inside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Group<'a> {
    /// The one inside no other held project.
    pub(crate) outermost: &'a Project,
    /// The outermost first, then the projects inside it, by root.
    pub(crate) members: Vec<&'a Project>,
}

/// The projects `places` holds, each in the group of the held project that holds it and lies inside no other; the
/// groups by the outermost's root.
pub(crate) fn groups<'a>(projects: &'a [Project], places: &Places) -> Vec<Group<'a>> {
    let mut held: Vec<&Project> = projects.iter().filter(|project| places.holds(&project.root)).collect();
    // By components: a folder's descendants come right after it, before its next sibling.
    held.sort_by(|first, second| first.root.cmp(&second.root));
    let mut groups: Vec<Group<'a>> = Vec::new();
    for project in held {
        match groups.last_mut() {
            Some(group) if project.root.starts_with(&group.outermost.root) => group.members.push(project),
            _ => groups.push(Group { outermost: project, members: vec![project] }),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use fresher_core::{KnownFolder, LastChange, Project, ProjectKind, SystemFacts, SystemPorts};
    use fresher_testing::{FakeInstallRecords, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

    use crate::place::Places;

    fn system(folders: FakeKnownFolders) -> SystemFacts {
        SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(folders),
            apps: Arc::new(FakeInstalledApps::default()),
            running: Arc::new(FakeRunningApps::default()),
            installs: Arc::new(FakeInstallRecords::default()),
        })
    }

    /// The places of a home at `/home`, with the system's apps folder at `/Applications`.
    fn places() -> Places {
        let folders = FakeKnownFolders::home_layout(Path::new("/home"))
            .with(KnownFolder::Applications, "/Applications")
            .with(KnownFolder::SystemApplications, "/System/Applications");
        Places::of(&system(folders)).unwrap()
    }

    fn project(root: &str) -> Project {
        Project {
            root: root.into(),
            kinds: vec![ProjectKind::Node],
            artifacts: Vec::new(),
            caches: Vec::new(),
            last_activity: LastChange::Unknown,
        }
    }

    fn roots<'a>(group: &crate::place::Group<'a>) -> Vec<&'a str> {
        group.members.iter().map(|member| member.root.to_str().unwrap()).collect()
    }

    mod negative {
        use std::path::Path;

        use fresher_testing::FakeKnownFolders;

        use super::{places, system};
        use crate::place::Places;

        #[test]
        fn the_home_and_its_ancestors_are_not_projects() {
            let places = places();

            for root in ["/home", "/", "/home/"] {
                assert!(!places.holds(Path::new(root)), "{root}");
            }
        }

        #[test]
        fn nothing_is_held_when_the_home_is_unknown() {
            assert_eq!(Places::of(&system(FakeKnownFolders::default())), None);
        }

        #[test]
        fn a_project_in_library_or_an_apps_folder_is_not_held() {
            let places = places();

            for root in [
                "/home/Library",
                "/home/Library/Application Support/X/app",
                "/home/Applications/tool",
                "/Applications/tool",
                "/System/Applications/tool",
            ] {
                assert!(!places.holds(Path::new(root)), "{root}");
            }
        }

        #[test]
        fn a_project_inside_an_app_bundle_is_not_held() {
            let places = places();

            for root in [
                "/opt/Tool.APP/Contents/Resources/app",
                "/home/Code/Tool.app/Contents/Resources/app",
                "/home/Code/tool.App",
            ] {
                assert!(!places.holds(Path::new(root)), "{root}");
            }
        }

        #[test]
        fn a_project_in_a_hidden_folder_of_the_home_is_not_held() {
            let places = places();

            for root in ["/home/.config/x", "/home/.cargo/registry/src/crate"] {
                assert!(!places.holds(Path::new(root)), "{root}");
            }
        }

        #[test]
        fn a_hidden_folder_deeper_down_is_held() {
            assert!(places().holds(Path::new("/home/Code/.archive/x")));
        }

        #[test]
        fn a_look_alike_prefix_is_not_inside() {
            let places = places();

            assert!(places.holds(Path::new("/home/Library2/x")));
            assert!(places.holds(Path::new("/home/Code/apps.application/x")));
            assert!(places.holds(Path::new("/homework/x")));
        }
    }

    mod positive {
        use super::{places, project, roots};
        use crate::place::groups;

        #[test]
        fn a_monorepo_is_one_group_under_its_outermost_project() {
            let projects = [
                project("/home/Code/shop/packages/web"),
                project("/home/Code/shop"),
                project("/home/Code/shop/packages/api"),
            ];

            let groups = groups(&projects, &places());

            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].outermost.root, std::path::Path::new("/home/Code/shop"));
            assert_eq!(
                roots(&groups[0]),
                ["/home/Code/shop", "/home/Code/shop/packages/api", "/home/Code/shop/packages/web"]
            );
        }

        #[test]
        fn two_sibling_projects_are_two_groups() {
            let projects =
                [project("/home/Code/shop-admin"), project("/home/Code/shop"), project("/home/Code/shop/web")];

            let groups = groups(&projects, &places());

            let all: Vec<Vec<&str>> = groups.iter().map(roots).collect();
            assert_eq!(all, [vec!["/home/Code/shop", "/home/Code/shop/web"], vec!["/home/Code/shop-admin"]]);
        }

        #[test]
        fn a_project_inside_a_left_out_project_is_its_own_outermost() {
            let projects = [project("/home"), project("/home/Code/app")];

            let groups = groups(&projects, &places());

            let all: Vec<Vec<&str>> = groups.iter().map(roots).collect();
            assert_eq!(all, [vec!["/home/Code/app"]]);
        }
    }
}
