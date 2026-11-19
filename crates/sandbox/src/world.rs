use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use fresher_core::{Access, BundleId, Capability, GuardedFolder, Receipt, RunningApp, SystemVersion, Timestamp};
use fresher_testing::{
    FakeAccess, FakeInstallRecords, FakeRunningApps, Stated, StatedFacts, StatedFlag, StatedKind, StatedOwner,
    time_at_age,
};
use serde::{Deserialize, Serialize};

use crate::case::{RawAccess, guarded_folder, is_bundle_name, is_pkg_name, is_plain};
use crate::{CaseFile, Sandbox, SandboxError};

/// The largest `world.toml` read: far more than any set of case files declares.
const MAX_WORLD_BYTES: u64 = 1024 * 1024;

/// Where Spotlight answers an app of `[[elsewhere]]`, below the sandbox: never written and never read.
const ELSEWHERE_FOLDER: &str = "Volumes/Elsewhere";

/// What the case files declare about the Mac beyond their files: the apps that are open, what Spotlight answers, what
/// the guarded folders allow, the installer receipts and the macOS version, the moment the sandbox was generated, and
/// the facts its entries state instead of having them (an old status change, an owner, a flag, a socket). Written as
/// `world.toml` at the sandbox's top; sandbox mode and the autotest take the running apps, Spotlight's answers, the
/// access, the clock's start and the stated facts from it, never from the real Mac.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    /// The generator's "now", in seconds since the Unix epoch: every age of the case files counts back from it.
    now: Option<u64>,
    running: Vec<RunningApp>,
    /// Whether Spotlight can be asked: on unless a case says `spotlight = "off"`.
    spotlight_on: bool,
    /// The apps Spotlight finds outside the apps folders, by name and bundle id, in the order of the case files.
    elsewhere: Vec<(String, BundleId)>,
    /// What the guarded folders allow, in the order of [`GuardedFolder`]; a folder no case states is absent.
    access: Vec<(GuardedFolder, Access)>,
    /// The macOS version the cases state; `None` when none does.
    macos: Option<SystemVersion>,
    /// The installer receipts: each package's file name and the moment it was installed, in seconds since the Unix
    /// epoch, in the order of the case files.
    receipts: Vec<(String, u64)>,
    /// The stated facts, by path relative to the sandbox, in the order of the case files.
    entries: Vec<(PathBuf, Stated)>,
}

impl Default for World {
    /// No moment, no running app, Spotlight on with nothing elsewhere, no access stated, no stated fact: a sandbox
    /// without `world.toml`.
    fn default() -> Self {
        Self {
            now: None,
            running: Vec::new(),
            spotlight_on: true,
            elsewhere: Vec::new(),
            access: Vec::new(),
            macos: None,
            receipts: Vec::new(),
            entries: Vec::new(),
        }
    }
}

/// `world.toml`, as written and read back.
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawWorld {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    now: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    spotlight: Option<RawSpotlight>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    macos: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    access: BTreeMap<String, RawAccess>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    running: Vec<RawRunning>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    elsewhere: Vec<RawElsewhere>,
    #[serde(default, rename = "receipt", skip_serializing_if = "Vec::is_empty")]
    receipts: Vec<RawReceipt>,
    #[serde(default, rename = "entry", skip_serializing_if = "Vec::is_empty")]
    entries: Vec<RawEntry>,
}

/// A `[[receipt]]`: an installer receipt of the sandbox's Mac, its moment in seconds since the Unix epoch.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawReceipt {
    package: String,
    installed: u64,
}

/// An `[[entry]]`: a path of the sandbox and the facts it states, its times in seconds since the Unix epoch.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    changed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accessed: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<StatedOwner>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    flags: BTreeSet<StatedFlag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    kind: Option<StatedKind>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawRunning {
    name: String,
    bundle_ids: Vec<String>,
}

/// An `[[elsewhere]]`: an app Spotlight finds outside the apps folders.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawElsewhere {
    name: String,
    bundle_id: String,
}

/// `spotlight`, written only when it is off.
#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RawSpotlight {
    Off,
}

impl World {
    /// The world of `files` generated at `now`, joined: every case's installed app declared `running = true` (its
    /// name and bundle id), then its `[[case.running]]`, its `[[case.elsewhere]]`, and every fact its files state
    /// (times counted back from `now`), in the order of the files and their cases; Spotlight is off when any case
    /// says so; a folder whose access two cases state differently takes the most restrictive (`denied`, then
    /// `not-asked`, then `granted`).
    pub fn of(files: &[CaseFile], now: SystemTime) -> Result<Self, SandboxError> {
        let mut running = Vec::new();
        let mut entries = Vec::new();
        let mut elsewhere = Vec::new();
        let mut spotlight_on = true;
        let mut access: BTreeMap<GuardedFolder, Access> = BTreeMap::new();
        let mut macos: Option<SystemVersion> = None;
        let mut receipts = Vec::new();
        for case in files.iter().flat_map(CaseFile::cases) {
            elsewhere.extend(case.elsewhere().iter().cloned());
            spotlight_on &= !case.spotlight_off();
            for &(folder, stated) in case.access() {
                let joined = access.entry(folder).or_insert(stated);
                if restrictiveness(stated) > restrictiveness(*joined) {
                    *joined = stated;
                }
            }
            if let Some(stated) = case.macos() {
                match macos {
                    Some(first) if first != stated => {
                        return Err(SandboxError::MacosVersions {
                            first: version_text(first),
                            second: version_text(stated),
                        });
                    }
                    _ => macos = Some(stated),
                }
            }
            for (package, age) in case.receipts() {
                // The age was checked when the case was read.
                let installed = time_at_age(age, now).and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok());
                receipts.push((package.clone(), installed.map_or(0, |since| since.as_secs())));
            }
            entries.extend(case.tree().stated(now).map_err(|source| SandboxError::Tree { source })?);
            // An app's bundle id was checked when its case was read.
            running.extend(case.running_apps().filter_map(|app| {
                let id = BundleId::new(app.bundle_id.as_str()).ok()?;
                Some(RunningApp { name: app.name.clone(), bundle_ids: vec![id] })
            }));
            running.extend(case.running_elsewhere().iter().cloned());
        }
        let now = now.duration_since(SystemTime::UNIX_EPOCH).ok().map(|since| since.as_secs());
        let access = access.into_iter().collect();
        Ok(Self { now, running, spotlight_on, elsewhere, access, macos, receipts, entries })
    }

    /// The world of `sandbox`, read from its `world.toml`, through no link; a missing file is an empty world. Anything
    /// but a file, a file over `MAX_WORLD_BYTES`, one that is not of the world's format, or one that names an app with
    /// no or an invalid bundle id, is refused.
    pub fn read(sandbox: &Sandbox) -> Result<Self, SandboxError> {
        let path = sandbox.world();
        let io = |source| SandboxError::Io { path: path.clone(), source };
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(io(error)),
            Ok(metadata) if metadata.is_symlink() => return Err(SandboxError::ThroughLink { path }),
            Ok(metadata) if !metadata.is_file() => {
                return Err(SandboxError::World { path, reason: "it is not a file".into() });
            }
            Ok(metadata) if metadata.len() > MAX_WORLD_BYTES => {
                let reason = format!("it is over {MAX_WORLD_BYTES} bytes");
                return Err(SandboxError::World { path, reason });
            }
            Ok(_) => {}
        }
        let text = fs::read_to_string(&path).map_err(io)?;
        let world = Self::parse(&text).map_err(|reason| SandboxError::World { path: path.clone(), reason })?;
        if let Some(entry) = world.entries.iter().find(|(entry, _)| !holds(sandbox.path(), entry)) {
            let reason = format!("it states facts of `{}`, which the sandbox does not hold", entry.0.display());
            return Err(SandboxError::World { path, reason });
        }
        Ok(world)
    }

    /// The running apps, in their declared order.
    pub fn running(&self) -> &[RunningApp] {
        &self.running
    }

    /// The moment the sandbox was generated, which its ages count back from; `None` for a world written before the
    /// world held it.
    pub fn now(&self) -> Option<SystemTime> {
        self.now.map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
    }

    /// Whether Spotlight can be asked in the sandbox: on unless a case says `spotlight = "off"`.
    pub const fn spotlight_on(&self) -> bool {
        self.spotlight_on
    }

    /// The apps Spotlight finds outside the apps folders, each by bundle id and at `<sandbox>/Volumes/Elsewhere/
    /// <name>.app`, below `sandbox` (the canonical sandbox folder): a path Spotlight only answers, never written and
    /// never read. The app's search is built from them and [`Self::spotlight_on`].
    pub fn elsewhere_apps(&self, sandbox: &Path) -> Vec<(BundleId, PathBuf)> {
        let folder = sandbox.join(ELSEWHERE_FOLDER);
        self.elsewhere.iter().map(|(name, id)| (id.clone(), folder.join(format!("{name}.app")))).collect()
    }

    /// Whether the world declares anything of Spotlight: an app found elsewhere, or Spotlight off.
    pub fn declares_spotlight(&self) -> bool {
        !self.spotlight_on || !self.elsewhere.is_empty()
    }

    /// Whether the world states what any guarded folder allows.
    pub fn declares_access(&self) -> bool {
        !self.access.is_empty()
    }

    /// The access the app asks in the sandbox whose home is `home`; `None` when the world states none. Every
    /// capability the system grants has a state: every file `Unknown`, a stated folder as stated, every other
    /// `NotAsked`. Each stated folder is guarded at `home/<Folder>`, and trying it answers `Granted` for `granted` and
    /// `not-asked` (the simulated prompt is accepted), `Denied` for `denied`.
    pub fn access(&self, home: &Path) -> Option<FakeAccess> {
        if self.access.is_empty() {
            return None;
        }
        let declared = |folder| self.access.iter().find(|(stated, _)| *stated == folder).map(|&(_, access)| access);
        let states: Vec<(Capability, Access)> = Capability::SYSTEM
            .into_iter()
            .map(|capability| {
                let access = match capability {
                    Capability::AllFiles => Access::Unknown,
                    Capability::Folder(folder) => declared(folder).unwrap_or(Access::NotAsked),
                    _ => Access::NotAsked,
                };
                (capability, access)
            })
            .collect();
        let fake = FakeAccess::with(&states).home(home);
        Some(self.access.iter().fold(fake, |fake, &(folder, access)| {
            let answer = if access == Access::Denied { Access::Denied } else { Access::Granted };
            fake.guard(folder_name(folder), Capability::Folder(folder)).answering(folder_name(folder), answer)
        }))
    }

    /// Whether the world states the macOS version or any installer receipt.
    pub fn declares_installs(&self) -> bool {
        self.macos.is_some() || !self.receipts.is_empty()
    }

    /// The installer receipts and the macOS version the app asks in the sandbox: the world's receipts and, when a case
    /// states one, its version; without one, the version fails ("the sandbox states no macOS version"), never the
    /// Mac's.
    pub fn install_records(&self) -> FakeInstallRecords {
        let receipts = self
            .receipts
            .iter()
            .map(|(package, installed)| Receipt {
                package_file: package.clone(),
                installed: Timestamp::from_secs(*installed),
            })
            .collect();
        let records = FakeInstallRecords::default().with_receipts(receipts);
        match self.macos {
            Some(version) => records.with_version(version.major, version.minor),
            None => records.failing_version("the sandbox states no macOS version"),
        }
    }

    /// The folders whose access the world states.
    pub fn stated_folders(&self) -> Vec<GuardedFolder> {
        self.access.iter().map(|&(folder, _)| folder).collect()
    }

    /// The places of the stated folders in the sandbox whose home is `home`: `home/<Folder>` of each.
    pub fn stated_places(&self, home: &Path) -> Vec<PathBuf> {
        self.access.iter().map(|&(folder, _)| home.join(folder_name(folder))).collect()
    }

    /// The running apps as the port the app asks: the same answer at every call.
    pub fn running_apps(&self) -> FakeRunningApps {
        FakeRunningApps::new(self.running.clone())
    }

    /// The facts the sandbox's entries state, at their paths below `sandbox` (the canonical sandbox folder): what
    /// sandbox mode's reader reports instead of the disk's.
    pub fn stated(&self, sandbox: &Path) -> StatedFacts {
        StatedFacts::under(sandbox, self.entries.iter().map(|(path, stated)| (path.as_path(), stated)))
    }

    /// The `world.toml` text.
    pub(crate) fn text(&self) -> Result<String, SandboxError> {
        let running = self
            .running
            .iter()
            .map(|app| RawRunning {
                name: app.name.clone(),
                bundle_ids: app.bundle_ids.iter().map(|id| id.as_str().to_owned()).collect(),
            })
            .collect();
        let entries = self
            .entries
            .iter()
            .map(|(path, stated)| RawEntry {
                path: path.clone(),
                changed: stated.changed.map(Timestamp::secs),
                accessed: stated.accessed.map(Timestamp::secs),
                owner: stated.owner,
                flags: stated.flags.clone(),
                kind: stated.kind,
            })
            .collect();
        let elsewhere = self
            .elsewhere
            .iter()
            .map(|(name, id)| RawElsewhere { name: name.clone(), bundle_id: id.as_str().to_owned() })
            .collect();
        let spotlight = (!self.spotlight_on).then_some(RawSpotlight::Off);
        let access = self
            .access
            .iter()
            .filter_map(|&(folder, access)| Some((Capability::Folder(folder).id().to_owned(), RawAccess::of(access)?)))
            .collect();
        let macos = self.macos.map(version_text);
        let receipts = self
            .receipts
            .iter()
            .map(|(package, installed)| RawReceipt { package: package.clone(), installed: *installed })
            .collect();
        let body = toml::to_string(&RawWorld {
            now: self.now,
            spotlight,
            macos,
            access,
            running,
            elsewhere,
            receipts,
            entries,
        })
        .map_err(|error| SandboxError::World { path: "world.toml".into(), reason: error.to_string() })?;
        // The facts', Spotlight's and the access's lines only when there are any: a world without them is written as
        // before.
        let spotlight = if self.declares_spotlight() {
            "# `spotlight = \"off\"` and each [[elsewhere]] are what Spotlight answers in sandbox mode: it cannot be asked,\n\
             # or it finds the app at Volumes/Elsewhere/<name>.app, which is never written.\n"
        } else {
            ""
        };
        let access = if self.declares_access() {
            "# [access] is what the guarded folders allow in sandbox mode: granted, denied, or not-asked, which is\n\
             # granted once its switch in Settings → Permissions is on.\n"
        } else {
            ""
        };
        let installs = if self.declares_installs() {
            "# `macos` and each [[receipt]] are what the system's version and installer receipts answer in sandbox mode.\n"
        } else {
            ""
        };
        let facts = if self.entries.is_empty() {
            ""
        } else {
            "# Each [[entry]] states facts of a path that the disk does not have (Finder never shows them).\n"
        };
        Ok(format!(
            "# The moment the sandbox was generated and the running apps the app sees in sandbox mode, written by the\n\
             # generator from the case files.\n{spotlight}{access}{installs}{facts}{body}"
        ))
    }

    fn parse(text: &str) -> Result<Self, String> {
        let raw: RawWorld = toml::from_str(text).map_err(|error| error.to_string())?;
        let running = raw
            .running
            .into_iter()
            .map(|app| {
                if app.bundle_ids.is_empty() {
                    return Err(format!("the running app \"{}\" has no bundle id", app.name));
                }
                let bundle_ids = app
                    .bundle_ids
                    .iter()
                    .map(|id| BundleId::new(id.as_str()).map_err(|_| format!("`{id}` is not a bundle id")))
                    .collect::<Result<_, _>>()?;
                Ok(RunningApp { name: app.name, bundle_ids })
            })
            .collect::<Result<_, String>>()?;
        let entries = raw
            .entries
            .into_iter()
            .map(|entry| {
                let at = entry.path.display().to_string();
                if !is_plain(&entry.path) {
                    return Err(format!("the entry `{at}` is not a plain path inside the sandbox"));
                }
                let stated = Stated {
                    changed: entry.changed.map(Timestamp::from_secs),
                    accessed: entry.accessed.map(Timestamp::from_secs),
                    owner: entry.owner,
                    flags: entry.flags,
                    kind: entry.kind,
                };
                if stated.is_empty() {
                    return Err(format!("the entry `{at}` states nothing"));
                }
                Ok((entry.path, stated))
            })
            .collect::<Result<_, String>>()?;
        let elsewhere = raw
            .elsewhere
            .into_iter()
            .map(|app| {
                if !is_bundle_name(&app.name) {
                    return Err(format!("the app found elsewhere {:?} is not named by one plain file name", app.name));
                }
                let id = BundleId::new(app.bundle_id.as_str())
                    .map_err(|_| format!("`{}` is not a bundle id", app.bundle_id))?;
                Ok((app.name, id))
            })
            .collect::<Result<_, String>>()?;
        let access = raw
            .access
            .into_iter()
            .map(|(key, value)| match guarded_folder(&key) {
                Some(folder) => Ok((folder, value.access())),
                None => {
                    Err(format!("`access` names `{key}`: only the guarded folders desktop, documents and downloads"))
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        let macos = raw
            .macos
            .map(|text| SystemVersion::parse(&text).ok_or_else(|| format!("`macos` is {text:?}, not a macOS version")))
            .transpose()?;
        let receipts = raw
            .receipts
            .into_iter()
            .map(|receipt| {
                if !is_pkg_name(&receipt.package) {
                    return Err(format!("the receipt `{}` is not one plain file name ending in .pkg", receipt.package));
                }
                Ok((receipt.package, receipt.installed))
            })
            .collect::<Result<_, String>>()?;
        let spotlight_on = raw.spotlight.is_none();
        Ok(Self { now: raw.now, running, spotlight_on, elsewhere, access, macos, receipts, entries })
    }
}

/// `15.4`: the version as a case file and `world.toml` write it.
fn version_text(version: SystemVersion) -> String {
    format!("{}.{}", version.major, version.minor)
}

/// How much `access` withholds: of two statements of one folder, the higher wins.
const fn restrictiveness(access: Access) -> u8 {
    match access {
        Access::Granted | Access::NotApplicable => 0,
        Access::NotAsked => 1,
        Access::Denied | Access::Unknown => 2,
    }
}

/// The name of `folder` in a home folder.
pub(crate) const fn folder_name(folder: GuardedFolder) -> &'static str {
    match folder {
        GuardedFolder::Desktop => "Desktop",
        GuardedFolder::Documents => "Documents",
        GuardedFolder::Downloads => "Downloads",
    }
}

/// Whether the folder `sandbox` holds an entry at `relative`, reached through no link; the entry may be a link.
fn holds(sandbox: &Path, relative: &Path) -> bool {
    let mut path = sandbox.to_path_buf();
    let mut names = relative.components().peekable();
    while let Some(name) = names.next() {
        path.push(name);
        match fs::symlink_metadata(&path) {
            // A folder on the way (never a link: `symlink_metadata` reads a link as no folder), or the entry itself.
            Ok(metadata) if names.peek().is_none() || metadata.is_dir() => {}
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    use fresher_core::Protection;
    use fresher_testing::FIXTURE_NOW;

    use crate::{CaseFile, Sandbox, World, generate};

    /// A case file whose cases declare `running` lines: a negative case with no app.
    fn case_file(feature: &str, running: &str) -> CaseFile {
        CaseFile::parse(&format!(
            "feature = \"{feature}\"\n[[case]]\nname = \"{feature} case\"\nkind = \"negative\"\n\
             [[case.file]]\npath = \"home/{feature}.bin\"\n{running}"
        ))
        .unwrap()
    }

    fn installed(path: &str, id: &str, name: &str, running: bool) -> String {
        format!("[[case.app]]\npath = \"{path}\"\nbundle-id = \"{id}\"\nname = \"{name}\"\nrunning = {running}\n")
    }

    fn elsewhere(name: &str, ids: &str) -> String {
        format!("[[case.running]]\nname = \"{name}\"\nbundle-ids = [{ids}]\n")
    }

    fn spotlight(name: &str, id: &str) -> String {
        format!("[[case.elsewhere]]\nname = \"{name}\"\nbundle-id = \"{id}\"\n")
    }

    /// A case file whose one case says `spotlight = "off"`.
    fn spotlight_off(feature: &str) -> CaseFile {
        CaseFile::parse(&format!(
            "feature = \"{feature}\"\n[[case]]\nname = \"{feature} case\"\nkind = \"negative\"\nspotlight = \"off\"\n\
             [[case.file]]\npath = \"home/{feature}.bin\"\n"
        ))
        .unwrap()
    }

    /// A case file whose one case states `access` (the inside of its inline table).
    fn with_access(feature: &str, access: &str) -> CaseFile {
        CaseFile::parse(&format!(
            "feature = \"{feature}\"\n[[case]]\nname = \"{feature} case\"\nkind = \"negative\"\n\
             access = {{ {access} }}\n[[case.file]]\npath = \"home/{feature}.bin\"\n"
        ))
        .unwrap()
    }

    /// A case file whose one case states `macos` (when not empty) and the receipts `[[case.receipt]]` of `receipts`.
    fn with_installs(feature: &str, macos: &str, receipts: &str) -> CaseFile {
        let macos = if macos.is_empty() { String::new() } else { format!("macos = \"{macos}\"\n") };
        CaseFile::parse(&format!(
            "feature = \"{feature}\"\n[[case]]\nname = \"{feature} case\"\nkind = \"negative\"\n{macos}\
             [[case.file]]\npath = \"home/{feature}.bin\"\n{receipts}"
        ))
        .unwrap()
    }

    const RECEIPT: &str = "[[case.receipt]]\npackage = \"Tool-3.1.pkg\"\ninstalled = \"10d\"\n";

    fn user(folder: &Path) -> u32 {
        use std::os::unix::fs::MetadataExt as _;
        fs::metadata(folder).unwrap().uid()
    }

    fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW)
    }

    /// `files` generated into a temporary folder, opened.
    fn generated(files: &[CaseFile]) -> (tempfile::TempDir, Sandbox) {
        let folder = tempfile::tempdir().unwrap();
        generate(files, folder.path(), now(), &Protection::default(), user(folder.path())).unwrap();
        let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
        (folder, sandbox)
    }

    fn names(world: &World) -> Vec<&str> {
        world.running().iter().map(|app| app.name.as_str()).collect()
    }

    /// Generates `files` again into the sandbox at `folder`.
    fn generate_into(folder: &Path, files: &[CaseFile]) {
        generate(files, folder, now(), &Protection::default(), user(folder)).unwrap();
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use fresher_core::{InstallRecords, RunningApps};

        use super::{RECEIPT, case_file, elsewhere, generated, installed, with_installs};
        use crate::{SandboxError, World};

        #[test]
        fn a_world_without_access_writes_no_access_table() {
            let file = case_file("open", &elsewhere("CLion", "\"com.jetbrains.CLion\""));
            let world = World::of(&[file], super::now()).unwrap();

            assert_eq!(
                world.text().unwrap(),
                format!(
                    "# The moment the sandbox was generated and the running apps the app sees in sandbox mode, written \
                     by the\n# generator from the case files.\nnow = {}\n\n[[running]]\nname = \"CLion\"\n\
                     bundle-ids = [\"com.jetbrains.CLion\"]\n",
                    fresher_testing::FIXTURE_NOW
                )
            );
            assert!(!world.declares_access());
            assert!(world.access(Path::new("/box/home")).is_none());
            assert_eq!(world.stated_places(Path::new("/box/home")), Vec::<std::path::PathBuf>::new());
        }

        #[test]
        fn two_different_macos_versions_are_refused() {
            let files = [with_installs("a", "15.4", ""), with_installs("b", "14.7", "")];

            let error = World::of(&files, super::now()).unwrap_err();

            assert!(
                matches!(&error, SandboxError::MacosVersions { first, second } if first == "15.4" && second == "14.7")
            );
            assert!(error.to_string().contains("15.4") && error.to_string().contains("14.7"));
            assert!(World::of(&[with_installs("a", "15.4", ""), with_installs("b", "15.4", "")], super::now()).is_ok());
        }

        #[test]
        fn a_world_without_installs_writes_neither() {
            let world = World::of(&[case_file("plain", "")], super::now()).unwrap();

            assert_eq!(
                world.text().unwrap(),
                format!(
                    "# The moment the sandbox was generated and the running apps the app sees in sandbox mode, written \
                     by the\n# generator from the case files.\nnow = {}\n",
                    fresher_testing::FIXTURE_NOW
                )
            );
            assert!(!world.declares_installs());
        }

        #[test]
        fn a_receipt_or_version_world_toml_cannot_hold_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for installs in [
                "macos = \"x\"\n",
                "[[receipt]]\npackage = \"a/b.pkg\"\ninstalled = 1\n",
                "[[receipt]]\npackage = \"Tool.dmg\"\ninstalled = 1\n",
                "[[receipt]]\npackage = \"Tool.pkg\"\n",
            ] {
                fs::write(sandbox.world(), installs).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{installs}: {error}");
            }
        }

        #[test]
        fn an_unstated_version_fails() {
            let world = World::of(&[with_installs("a", "", RECEIPT)], super::now()).unwrap();

            let error = world.install_records().system_version().unwrap_err();

            assert_eq!(error.reason, "the sandbox states no macOS version");
        }

        #[test]
        fn an_unknown_access_key_in_world_toml_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for access in [
                "[access]\nall-files = \"granted\"\n",
                "[access]\nx = \"denied\"\n",
                "[access]\ndownloads = \"yes\"\n",
                "[access]\ndownloads = \"unknown\"\n",
            ] {
                fs::write(sandbox.world(), access).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{access}: {error}");
            }
        }

        #[test]
        fn a_sandbox_with_no_world_file_has_no_running_app_and_no_now() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::default());
            assert_eq!(world.now(), None);
            assert_eq!(world.running_apps().list(), Ok(Vec::new()));
        }

        #[test]
        fn an_app_that_is_not_running_is_not_in_the_world() {
            let file = case_file("apps", &installed("Applications/Tool.app", "a.tool", "Tool", false));

            assert_eq!(World::of(&[file], super::now()).unwrap().running(), []);
        }

        #[test]
        fn a_world_file_with_a_now_that_is_not_a_number_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for now in ["now = \"today\"\n", "now = -1\n"] {
                fs::write(sandbox.world(), now).unwrap();

                assert!(matches!(World::read(&sandbox), Err(SandboxError::World { .. })), "{now}");
            }
        }

        #[test]
        fn a_world_file_with_no_or_an_invalid_bundle_id_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for running in [
                "[[running]]\nname = \"CLion\"\nbundle-ids = []\n",
                "[[running]]\nname = \"CLion\"\n",
                "[[running]]\nname = \"CLion\"\nbundle-ids = [\"not an id\"]\n",
                "[[running]]\nname = \"CLion\"\nbundle-ids = [\"a.b\"]\ncolour = \"red\"\n",
            ] {
                fs::write(sandbox.world(), running).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{running}: {error}");
            }
        }

        #[test]
        fn a_world_file_that_is_a_folder_or_too_large_is_refused() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();
            fs::create_dir(sandbox.world()).unwrap();

            assert!(matches!(World::read(&sandbox), Err(SandboxError::World { .. })));

            let (_large_folder, large) = generated(&[case_file("plain", "")]);
            fs::OpenOptions::new().write(true).open(large.world()).unwrap().set_len(1024 * 1024 + 1).unwrap();

            assert!(matches!(World::read(&large), Err(SandboxError::World { .. })));
        }

        #[test]
        fn an_entry_outside_the_sandbox_or_stating_nothing_or_unknown_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for entry in [
                "[[entry]]\npath = \"../out\"\nowner = \"system\"\n",
                "[[entry]]\npath = \"/etc/hosts\"\nowner = \"system\"\n",
                "[[entry]]\npath = \"home/plain.bin\"\n",
                "[[entry]]\npath = \"home/plain.bin\"\nowner = \"root\"\n",
                "[[entry]]\npath = \"home/plain.bin\"\nflags = [\"sticky\"]\n",
                "[[entry]]\npath = \"home/plain.bin\"\nchanged = \"30d\"\n",
                "[[entry]]\npath = \"home/plain.bin\"\nkind = \"socket\"\ncolour = 1\n",
            ] {
                fs::write(sandbox.world(), entry).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{entry}: {error}");
            }
        }

        #[test]
        fn a_stated_fact_on_a_path_the_sandbox_does_not_hold_is_refused() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            symlink(folder.path().join("home"), folder.path().join("linked")).unwrap();
            for path in ["home/missing.bin", "linked/plain.bin", "home/plain.bin/inside"] {
                fs::write(sandbox.world(), format!("[[entry]]\npath = \"{path}\"\nowner = \"system\"\n")).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{path}: {error}");
            }
        }

        #[test]
        fn a_world_file_with_a_bad_elsewhere_app_or_spotlight_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for world in [
                "[[elsewhere]]\nname = \"Far\"\nbundle-id = \"not an id\"\n",
                "[[elsewhere]]\nname = \"\"\nbundle-id = \"com.example.Far\"\n",
                "[[elsewhere]]\nname = \"../Far\"\nbundle-id = \"com.example.Far\"\n",
                "[[elsewhere]]\nname = \"Far\"\n",
                "[[elsewhere]]\nname = \"Far\"\nbundle-id = \"com.example.Far\"\ncolour = 1\n",
                "spotlight = \"on\"\n",
                "spotlight = false\n",
            ] {
                fs::write(sandbox.world(), world).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{world}: {error}");
            }
        }

        #[test]
        fn a_world_without_spotlight_answers_is_written_as_before() {
            let world = World::of(&[case_file("plain", "")], super::now()).unwrap();

            assert_eq!(
                world.text().unwrap(),
                format!(
                    "# The moment the sandbox was generated and the running apps the app sees in sandbox mode, written \
                     by the\n# generator from the case files.\nnow = {}\n",
                    fresher_testing::FIXTURE_NOW
                )
            );
            assert!(world.spotlight_on());
            assert!(!world.declares_spotlight());
            assert_eq!(world.elsewhere_apps(Path::new("/box")), []);
        }

        #[test]
        fn a_sandbox_with_no_world_file_has_spotlight_on_and_nothing_elsewhere() {
            let world = World::default();

            assert!(world.spotlight_on());
            assert_eq!(world.elsewhere_apps(Path::new("/box")), []);
        }

        #[test]
        fn a_world_file_that_is_a_link_is_refused() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            let elsewhere = folder.path().join("home/elsewhere.toml");
            fs::write(&elsewhere, "").unwrap();
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();
            symlink(&elsewhere, sandbox.world()).unwrap();

            assert!(matches!(World::read(&sandbox), Err(SandboxError::ThroughLink { .. })));
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::{
            Access, AccessProbe, Capability, GuardedFolder, GuardedPlace, InstallRecords, Receipt, RunningApps,
            SystemVersion, Timestamp,
        };
        use fresher_testing::{FIXTURE_NOW, StatedFlag, StatedKind, StatedOwner};

        use std::path::{Path, PathBuf};

        use super::{
            RECEIPT, case_file, elsewhere, generated, installed, names, spotlight, spotlight_off, with_access,
            with_installs,
        };
        use crate::World;

        const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);
        const DOCUMENTS: Capability = Capability::Folder(GuardedFolder::Documents);
        const DOWNLOADS: Capability = Capability::Folder(GuardedFolder::Downloads);

        #[test]
        fn the_most_restrictive_statement_wins() {
            let denied = [with_access("a", "downloads = \"granted\""), with_access("b", "downloads = \"denied\"")];
            let not_asked = [with_access("a", "desktop = \"granted\""), with_access("b", "desktop = \"not-asked\"")];
            let reversed = [with_access("a", "downloads = \"denied\""), with_access("b", "downloads = \"granted\"")];

            let home = Path::new("/box/home");
            let state = |files: &[crate::CaseFile], capability| {
                World::of(files, super::now()).unwrap().access(home).unwrap().status().access(&capability)
            };

            assert_eq!(state(&denied, DOWNLOADS), Access::Denied);
            assert_eq!(state(&reversed, DOWNLOADS), Access::Denied);
            assert_eq!(state(&not_asked, DESKTOP), Access::NotAsked);
        }

        #[test]
        fn installs_round_trip_through_world_toml() {
            let files = [with_installs("a", "15.4", RECEIPT), case_file("b", "")];
            let (_folder, sandbox) = generated(&files);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::of(&files, super::now()).unwrap());
            assert!(world.declares_installs());
            let text = fs::read_to_string(sandbox.world()).unwrap();
            assert!(text.contains("macos = \"15.4\"\n"), "{text}");
            assert!(text.contains("[[receipt]]\npackage = \"Tool-3.1.pkg\"\ninstalled = "), "{text}");
        }

        #[test]
        fn install_records_answer_the_stated_version_and_receipts() {
            let world = World::of(&[with_installs("a", "15.4", RECEIPT)], super::now()).unwrap();

            let records = world.install_records();

            assert_eq!(records.system_version(), Ok(SystemVersion { major: 15, minor: 4 }));
            assert_eq!(
                records.receipts(),
                Ok(vec![Receipt {
                    package_file: "Tool-3.1.pkg".into(),
                    installed: Timestamp::from_secs(FIXTURE_NOW - 10 * 24 * 3600)
                }])
            );
        }

        #[test]
        fn access_round_trips_through_world_toml() {
            let files = [with_access("a", "downloads = \"granted\", desktop = \"denied\""), case_file("b", "")];
            let (_folder, sandbox) = generated(&files);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::of(&files, super::now()).unwrap());
            assert!(world.declares_access());
            let text = fs::read_to_string(sandbox.world()).unwrap();
            assert!(text.contains("[access]\ndesktop = \"denied\"\ndownloads = \"granted\"\n"), "{text}");
            assert!(text.contains("# [access] is what the guarded folders allow in sandbox mode"), "{text}");
        }

        #[test]
        fn access_answers_every_capability() {
            let world = World::of(&[with_access("a", "downloads = \"not-asked\"")], super::now()).unwrap();
            let home = Path::new("/box/home");

            let access = world.access(home).unwrap();

            let status = access.status();
            assert_eq!(status.capabilities.len(), Capability::SYSTEM.len());
            assert_eq!(status.access(&Capability::AllFiles), Access::Unknown);
            assert_eq!(status.access(&DOCUMENTS), Access::NotAsked);
            assert_eq!(status.access(&DOWNLOADS), Access::NotAsked);
            assert_eq!(status.access(&Capability::OtherAppsData), Access::NotAsked);
            let places = access.guarded_places(&[home.to_path_buf()]);
            assert_eq!(places, [GuardedPlace { path: home.join("Downloads"), capability: DOWNLOADS }]);
            assert_eq!(access.try_place(&places[0]), Access::Granted, "the simulated prompt is accepted");

            let denied = World::of(&[with_access("a", "downloads = \"denied\"")], super::now()).unwrap();
            let denied = denied.access(home).unwrap();
            assert_eq!(denied.status().access(&DOWNLOADS), Access::Denied);
            assert_eq!(denied.try_place(&denied.guarded_places(&[home.to_path_buf()])[0]), Access::Denied);

            let granted = World::of(&[with_access("a", "downloads = \"granted\"")], super::now()).unwrap();
            let granted = granted.access(home).unwrap();
            assert_eq!(granted.status().access(&DOWNLOADS), Access::Granted);
            assert_eq!(granted.try_place(&granted.guarded_places(&[home.to_path_buf()])[0]), Access::Granted);
        }

        #[test]
        fn stated_places_are_the_stated_folders_under_the_home() {
            let file = with_access("a", "downloads = \"granted\", documents = \"denied\"");
            let world = World::of(&[file], super::now()).unwrap();

            let places = world.stated_places(Path::new("/box/home"));

            assert_eq!(places, [PathBuf::from("/box/home/Documents"), PathBuf::from("/box/home/Downloads")]);
        }

        #[test]
        fn an_installed_app_declared_running_is_in_the_world_with_its_bundle_id() {
            let file =
                case_file("apps", &installed("Applications/WebStorm.app", "com.jetbrains.WebStorm", "WebStorm", true));
            let (_folder, sandbox) = generated(&[file]);

            let world = World::read(&sandbox).unwrap();

            let listed = world.running_apps().list().unwrap();
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].name, "WebStorm");
            assert_eq!(listed[0].bundle_ids[0].as_str(), "com.jetbrains.WebStorm");
        }

        #[test]
        fn a_running_entry_is_in_the_world_with_its_bundle_ids() {
            let file = case_file("open", &elsewhere("CLion", "\"com.jetbrains.CLion\", \"a.helper\""));
            let (_folder, sandbox) = generated(&[file]);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(names(&world), ["CLion"]);
            let ids: Vec<&str> = world.running()[0].bundle_ids.iter().map(fresher_core::BundleId::as_str).collect();
            assert_eq!(ids, ["com.jetbrains.CLion", "a.helper"]);
        }

        #[test]
        fn two_case_files_join_their_worlds() {
            let first = case_file("first", &installed("Applications/Tool.app", "a.tool", "Tool", true));
            let second = case_file("second", &elsewhere("CLion", "\"com.jetbrains.CLion\""));
            let (_folder, sandbox) = generated(&[first, second]);

            assert_eq!(names(&World::read(&sandbox).unwrap()), ["Tool", "CLion"]);
        }

        #[test]
        fn a_world_file_left_by_an_earlier_run_is_reset() {
            let open = case_file("box", &elsewhere("CLion", "\"com.jetbrains.CLion\""));
            let (folder, sandbox) = generated(&[open]);

            super::generate_into(folder.path(), &[case_file("box", "")]);

            let world = World::read(&sandbox).unwrap();
            assert_eq!(world.running(), []);
            assert_eq!(world.now(), Some(super::now()));
            assert!(fs::read_to_string(sandbox.world()).unwrap().starts_with('#'));
        }

        #[test]
        fn the_world_holds_the_moment_the_sandbox_was_generated() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world.now(), Some(super::now()));
            let text = fs::read_to_string(sandbox.world()).unwrap();
            assert!(text.contains(&format!("now = {FIXTURE_NOW}")), "{text}");
        }

        #[test]
        fn the_world_holds_the_stated_facts_and_reads_them_back() {
            let facts =
                "owner = \"system\"\nchanged = \"30d\"\naccessed = \"-1h\"\nflags = [\"restricted\", \"user-lock\"]\n";
            let file =
                case_file("facts", &format!("[[case.file]]\npath = \"private/tmp/sock\"\nkind = \"pipe\"\n{facts}"));
            let (_folder, sandbox) = generated(std::slice::from_ref(&file));

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::of(&[file], super::now()).unwrap());
            let stated = world.stated(sandbox.path());
            assert_eq!(stated.len(), 1);
            let sock = stated.get(&sandbox.path().join("private/tmp/sock")).unwrap();
            assert_eq!(sock.owner, Some(StatedOwner::System));
            assert_eq!(sock.kind, Some(StatedKind::Pipe));
            assert_eq!(sock.changed, Some(Timestamp::from_secs(FIXTURE_NOW - 30 * 86_400)));
            assert_eq!(sock.accessed, Some(Timestamp::from_secs(FIXTURE_NOW + 3_600)));
            assert_eq!(sock.flags, [StatedFlag::Restricted, StatedFlag::UserLock].into());
            assert!(fs::symlink_metadata(sandbox.path().join("private/tmp/sock")).unwrap().is_file());
        }

        /// The link itself is the entry, so the world holds it; a path through it is still refused.
        #[test]
        fn a_stated_link_is_held_and_read_back() {
            let link = "[[case.symlink]]\npath = \"private/tmp/link\"\ntarget = \"../../home\"\nchanged = \"30d\"\n";
            let file = case_file("link", link);
            let (_folder, sandbox) = generated(std::slice::from_ref(&file));

            let stated = World::read(&sandbox).unwrap().stated(sandbox.path());

            let link = stated.get(&sandbox.path().join("private/tmp/link")).unwrap();
            assert_eq!(link.changed, Some(Timestamp::from_secs(FIXTURE_NOW - 30 * 86_400)));
        }

        #[test]
        fn two_case_files_join_their_stated_facts() {
            let first = case_file("first", "owner = \"other-user\"\n");
            let second =
                case_file("second", "[[case.dir]]\npath = \"private/var/folders/xx/sandbox/T/d\"\nchanged = \"2h\"\n");
            let (_folder, sandbox) = generated(&[first, second]);

            let stated = World::read(&sandbox).unwrap().stated(sandbox.path());

            assert_eq!(stated.len(), 2);
            assert_eq!(stated.get(&sandbox.path().join("home/first.bin")).unwrap().owner, Some(StatedOwner::OtherUser));
            let folder = stated.get(&sandbox.path().join("private/var/folders/xx/sandbox/T/d")).unwrap();
            assert_eq!(folder.changed, Some(Timestamp::from_secs(FIXTURE_NOW - 7_200)));
        }

        #[test]
        fn a_world_with_spotlight_off_and_an_app_elsewhere_reads_back_equal() {
            let files = [case_file("far", &spotlight("Far Away", "com.example.Far")), spotlight_off("off")];
            let (_folder, sandbox) = generated(&files);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::of(&files, super::now()).unwrap());
            assert!(!world.spotlight_on());
            assert!(world.declares_spotlight());
            let text = fs::read_to_string(sandbox.world()).unwrap();
            assert!(text.contains("spotlight = \"off\"\n"), "{text}");
            assert!(text.contains("[[elsewhere]]\nname = \"Far Away\"\nbundle-id = \"com.example.Far\"\n"), "{text}");
        }

        #[test]
        fn the_apps_elsewhere_are_answered_below_the_sandbox_and_never_written() {
            let file = case_file(
                "far",
                &format!("{}{}", spotlight("Far", "com.example.Far"), spotlight("Other", "com.example.Other")),
            );
            let (_folder, sandbox) = generated(&[file]);

            let world = World::read(&sandbox).unwrap();

            let answered = world.elsewhere_apps(Path::new("/box"));
            let apps: Vec<(&str, PathBuf)> = answered.iter().map(|(id, path)| (id.as_str(), path.clone())).collect();
            assert_eq!(
                apps,
                [
                    ("com.example.Far", PathBuf::from("/box/Volumes/Elsewhere/Far.app")),
                    ("com.example.Other", PathBuf::from("/box/Volumes/Elsewhere/Other.app")),
                ]
            );
            assert!(world.spotlight_on());
            assert!(!sandbox.path().join("Volumes").exists());
        }

        #[test]
        fn the_world_file_reads_back_as_written() {
            let apps = format!(
                "{}{}",
                installed("Applications/A.app", "a.a", "A \\\"quoted\\\"", true),
                elsewhere("B", "\"b.b\"")
            );
            let world = World::of(&[case_file("box", &apps)], super::now()).unwrap();

            let parsed = World::parse(&world.text().unwrap()).unwrap();

            assert_eq!(parsed, world);
            assert_eq!(parsed.running_apps().list().unwrap().len(), 2);
        }
    }
}
