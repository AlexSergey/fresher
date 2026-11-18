//! The owners of the places of a committed state: each installed app's data folders, found by its bundle id under
//! the known folders of app data, its bundle when a root holds it, and the Trash. Worked out once at the commit, with the
//! scan's facts of the system; nothing is written anywhere.

use std::collections::HashMap;
use std::iter;

use fresher_core::{AppRef, BundleId, ChildOrder, DiskState, InstalledApp, KnownFolder, NodeId, SystemFacts};

/// The known folders whose first-level children are named after the app they belong to.
const APP_DATA: [KnownFolder; 7] = [
    KnownFolder::AppCaches,
    KnownFolder::AppSupport,
    KnownFolder::Containers,
    KnownFolder::GroupContainers,
    KnownFolder::Preferences,
    KnownFolder::Logs,
    KnownFolder::SavedState,
];

/// The places of one state that belong to an installed app, and the Trash.
#[derive(Debug, Default)]
pub(crate) struct AppPlaces {
    /// The installed apps; `None` when they could not be listed: the map then names no app.
    apps: Option<Vec<AppRef>>,
    /// Each installed app's id and its helpers', in lower case: the app's position in `apps`.
    ids: HashMap<String, usize>,
    /// The data folders and bundles found, each with its app's position in `apps`.
    places: Vec<(NodeId, usize)>,
    trash: Option<NodeId>,
}

impl AppPlaces {
    /// The places of `state` by the facts of the scan that committed it. The installed apps are listed here when no
    /// module asked for them; a listing that fails leaves the apps out (logged), never the rest.
    pub(crate) fn of(state: &dyn DiskState, system: &SystemFacts) -> Self {
        let trash = system.locate(KnownFolder::Trash).and_then(|path| state.find(&path));
        let installed = match system.installed_apps() {
            Ok(installed) => installed,
            Err(error) => {
                tracing::warn!(%error, "the storage map names no app: the installed apps could not be listed");
                return Self { trash, ..Self::default() };
            }
        };
        let ids = ids(installed);
        let mut places = Vec::new();
        for folder in APP_DATA {
            let Some(node) = system.locate(folder).and_then(|path| state.find(&path)) else { continue };
            let Some(page) = state.children(node, ChildOrder::ByName, 0, u32::MAX) else { continue };
            for child in page.nodes {
                if let Some(app) = named_after(installed, &ids, &child.name) {
                    places.push((child.id, app));
                }
            }
        }
        for (position, app) in installed.iter().enumerate() {
            if let Some(bundle) = state.find(&app.path) {
                places.push((bundle, position));
            }
        }
        let apps = installed
            .iter()
            .map(|app| AppRef { id: app.bundle_id.clone(), name: app.name.clone(), installed: true })
            .collect();
        Self { apps: Some(apps), ids, places, trash }
    }
}

impl AppPlaces {
    /// The app `id` names: the installed app whose own id or helper's it is (its own name wins), else `name`, or the
    /// id itself when none is given, no longer installed. `None` when the installed apps could not be listed.
    pub(crate) fn app(&self, id: &BundleId, name: Option<&str>) -> Option<AppRef> {
        let apps = self.apps.as_ref()?;
        let installed = self.ids.get(id.folded()).and_then(|&position| apps.get(position));
        Some(installed.cloned().unwrap_or_else(|| AppRef {
            id: id.clone(),
            name: name.map_or_else(|| id.to_string(), str::to_owned),
            installed: false,
        }))
    }

    /// The data folders and bundles found, each with the installed app it belongs to.
    pub(crate) fn data(&self) -> impl Iterator<Item = (NodeId, &AppRef)> + '_ {
        let apps = self.apps.as_deref().unwrap_or_default();
        self.places.iter().filter_map(|&(node, position)| Some((node, apps.get(position)?)))
    }

    /// The node of the user's Trash; `None` when no root holds it.
    pub(crate) fn trash(&self) -> Option<NodeId> {
        self.trash
    }
}

/// Every id of `installed`, the app's own and its helpers', in lower case, with the app's position; an id two apps
/// share stays with the first.
fn ids(installed: &[InstalledApp]) -> HashMap<String, usize> {
    let mut ids = HashMap::new();
    for (position, app) in installed.iter().enumerate() {
        for id in iter::once(&app.bundle_id).chain(&app.helpers) {
            ids.entry(id.folded().to_owned()).or_insert(position);
        }
    }
    ids
}

/// The installed app `name` is named after (`BundleId::names`): the longest of the ids that name it, so a helper's
/// own folder goes to the helper's app.
fn named_after(installed: &[InstalledApp], ids: &HashMap<String, usize>, name: &str) -> Option<usize> {
    let lower = name.to_ascii_lowercase();
    let ends = lower.match_indices('.').map(|(at, _)| at).chain(iter::once(lower.len()));
    let mut ends: Vec<usize> = ends.collect();
    ends.reverse();
    ends.into_iter().find_map(|end| {
        let position = *ids.get(lower.get(..end)?)?;
        let app = installed.get(position)?;
        iter::once(&app.bundle_id).chain(&app.helpers).any(|id| id.names(name)).then_some(position)
    })
}

#[cfg(test)]
#[path = "attribution_tests.rs"]
mod tests;
