//! What the docker module's tests share: the homes' loader and every run of the module on them. The apps installed and
//! running and the runtime's answers are given to the module as fakes; facts the disk cannot be made to show (a socket)
//! are stated by the home.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::InstalledApp;
use fresher_module_docker::DockerModule;
use fresher_testing::{
    FIXTURE_NOW, FakeCommandRunner, FakeInstalledApps, FakeRunningApps, FixtureTree, ModuleRun, Ports, installed_app,
    run_module, running_app,
};

/// The repository's `fixtures` folder.
pub(crate) fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// The home `fixtures/homes/<name>.toml`.
pub(crate) fn home(name: &str) -> FixtureTree {
    FixtureTree::from_toml(&fixtures().join("homes").join(format!("{name}.toml"))).unwrap()
}

pub(crate) fn orbstack() -> InstalledApp {
    installed_app("dev.kdrag0n.MacVirt", "OrbStack", "2.2.5").unwrap()
}

pub(crate) fn desktop() -> InstalledApp {
    installed_app("com.docker.docker", "Docker", "4.0").unwrap()
}

/// The only call the module may make to `app`'s `OrbStack`, as `FakeCommandRunner` keys it; built from the app's path.
pub(crate) fn orbstack_read() -> String {
    let program = orbstack().path.join("Contents/MacOS/xbin/docker");
    format!("{} --context orbstack system df --format json", program.display())
}

/// The design's answer: something unused in every kind but the stopped containers.
pub(crate) const ANSWER: &str = r#"{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}
"#;

/// Another answer: nothing unused in images, volumes and build cache, one stopped container.
pub(crate) const ANSWER_CONTAINER: &str = r#"{"Active":"1","Reclaimable":"0B (0%)","Size":"2GB","TotalCount":"1","Type":"Images"}
{"Active":"0","Reclaimable":"3MB (100%)","Size":"3MB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"0B","Size":"1MB","TotalCount":"1","Type":"Local Volumes"}
{"Active":"1","Reclaimable":"0B","Size":"1MB","TotalCount":"1","Type":"Build Cache"}
"#;

/// A third answer: several stopped containers and one unused image.
pub(crate) const ANSWER_MANY: &str = r#"{"Active":"0","Reclaimable":"1GB (100%)","Size":"1GB","TotalCount":"1","Type":"Images"}
{"Active":"1","Reclaimable":"3MB (75%)","Size":"4MB","TotalCount":"4","Type":"Containers"}
{"Active":"0","Reclaimable":"1MB (100%)","Size":"1MB","TotalCount":"3","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"0B","Size":"0B","TotalCount":"0","Type":"Build Cache"}
"#;

/// What a scan was given besides the home.
pub(crate) struct Setup {
    pub(crate) apps: FakeInstalledApps,
    pub(crate) running: FakeRunningApps,
    pub(crate) commands: FakeCommandRunner,
    pub(crate) restored: bool,
    pub(crate) avoid: Vec<PathBuf>,
}

impl Setup {
    /// Nothing installed, nothing running, no answer, a walked scan, every place read.
    pub(crate) fn none() -> Self {
        Self {
            apps: FakeInstalledApps::new(Vec::new()),
            running: FakeRunningApps::default(),
            commands: FakeCommandRunner::default(),
            restored: false,
            avoid: Vec::new(),
        }
    }

    /// `app` installed.
    pub(crate) fn installed(app: InstalledApp) -> Self {
        Self { apps: FakeInstalledApps::new(vec![app]), ..Self::none() }
    }

    /// `app` installed and open.
    pub(crate) fn open(app: InstalledApp) -> Self {
        let name = app.name.clone();
        let id = app.bundle_id.as_str().to_owned();
        Self { running: FakeRunningApps::new(vec![running_app(&name, &[&id]).unwrap()]), ..Self::installed(app) }
    }

    /// `OrbStack` installed and open, its read answered with `stdout`.
    pub(crate) fn orbstack_says(stdout: &str) -> Self {
        Self { commands: FakeCommandRunner::default().answering(&orbstack_read(), stdout), ..Self::open(orbstack()) }
    }

    pub(crate) fn commands(self, commands: FakeCommandRunner) -> Self {
        Self { commands, ..self }
    }

    pub(crate) fn restored(self) -> Self {
        Self { restored: true, ..self }
    }

    pub(crate) fn running(self, running: FakeRunningApps) -> Self {
        Self { running, ..self }
    }

    pub(crate) fn avoiding(self, place: &str) -> Self {
        Self { avoid: vec![PathBuf::from(place)], ..self }
    }
}

/// What a scan did: the module's run and every command it ran, in order.
pub(crate) struct Scan {
    pub(crate) run: ModuleRun,
    pub(crate) calls: Vec<String>,
}

/// The home `name` scanned with `setup`, the clock at `FIXTURE_NOW`.
pub(crate) fn scan(name: &str, setup: Setup) -> Scan {
    let tree = home(name);
    let ports = Ports {
        apps: Arc::new(setup.apps),
        running: Arc::new(setup.running),
        commands: setup.commands,
        restored: setup.restored,
        avoid: setup.avoid,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&DockerModule::new().unwrap(), &tree, &ports).unwrap();
    Scan { run, calls: ports.commands.calls() }
}

/// The silent home with nothing installed.
pub(crate) fn silent() -> Scan {
    scan("docker-silent", Setup::none())
}

/// The silent home with Docker Desktop installed and closed.
pub(crate) fn silent_with_desktop() -> Scan {
    scan("docker-silent", Setup::installed(desktop()))
}

/// The stopped home with Docker Desktop installed and closed.
pub(crate) fn closed() -> Scan {
    scan("docker-stopped", Setup::installed(desktop()))
}

/// The home without a socket, `OrbStack` installed and open.
pub(crate) fn no_socket() -> Scan {
    scan("docker-stopped-no-socket", Setup::open(orbstack()))
}

/// The stopped home, `OrbStack` installed and open, restored.
pub(crate) fn restored() -> Scan {
    scan("docker-stopped", Setup::open(orbstack()).restored())
}

/// The stopped home, `OrbStack` installed with the running apps failing.
pub(crate) fn unknown() -> Scan {
    scan("docker-stopped", Setup::open(orbstack()).running(FakeRunningApps::failing("no process list")))
}

/// The answered home, `OrbStack` open and saying `stdout`.
pub(crate) fn answered_with(stdout: &str) -> Scan {
    scan("docker-answered", Setup::orbstack_says(stdout))
}

/// The answered home, `OrbStack` open and saying the design's answer.
pub(crate) fn answered() -> Scan {
    answered_with(ANSWER)
}

/// The answered home, `OrbStack` open and exiting with status 1.
pub(crate) fn no_answer() -> Scan {
    scan("docker-answered", Setup::open(orbstack()).commands(FakeCommandRunner::default().exiting(&orbstack_read(), 1)))
}

/// The answered home, `OrbStack` open and saying `garbage`.
pub(crate) fn unreadable() -> Scan {
    answered_with("garbage")
}

/// The answered home with the disk file's place avoided.
pub(crate) fn unseen() -> Scan {
    scan("docker-answered", Setup::orbstack_says(ANSWER).avoiding("Library/Group Containers"))
}

/// The left home, no app installed.
pub(crate) fn left() -> Scan {
    scan("docker-left", Setup::none())
}

/// Every run, for the contract and the checks that hold for all: the three answers, so that every part's texts are
/// used.
pub(crate) fn every_scan() -> Vec<Scan> {
    vec![
        silent(),
        silent_with_desktop(),
        closed(),
        no_socket(),
        restored(),
        unknown(),
        answered(),
        answered_with(ANSWER_CONTAINER),
        answered_with(ANSWER_MANY),
        no_answer(),
        unreadable(),
        unseen(),
        left(),
    ]
}
