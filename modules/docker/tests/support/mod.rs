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

/// The only call the module may make to `app`'s `OrbStack` in the home at `home`, as `FakeCommandRunner` keys it:
/// built from the app's path and the home's socket, with the system's empty folder as its config.
pub(crate) fn orbstack_read(home: &Path) -> String {
    let program = orbstack().path.join("Contents/MacOS/xbin/docker");
    let socket = home.join(".orbstack/run/docker.sock");
    format!(
        "{} --host unix://{} --config /private/var/empty system df --format json",
        program.display(),
        socket.display()
    )
}

/// How `OrbStack`'s read answers, if it is asked.
#[derive(Clone)]
pub(crate) enum Reply {
    Says(String),
    Exits(i32),
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
    /// The answer of `OrbStack`'s read; `None`: no command is answered.
    pub(crate) reply: Option<Reply>,
    pub(crate) restored: bool,
    pub(crate) avoid: Vec<PathBuf>,
}

impl Setup {
    /// Nothing installed, nothing running, no answer, a walked scan, every place read.
    pub(crate) fn none() -> Self {
        Self {
            apps: FakeInstalledApps::new(Vec::new()),
            running: FakeRunningApps::default(),
            reply: None,
            restored: false,
            avoid: Vec::new(),
        }
    }

    /// `app` installed, the CLI of the runtime it is, if any, signed by that runtime's developer.
    pub(crate) fn installed(app: InstalledApp) -> Self {
        let cli = match app.bundle_id.as_str() {
            "com.docker.docker" => Some(("Contents/Resources/bin/docker", "9BNSXJN65R")),
            "dev.kdrag0n.MacVirt" => Some(("Contents/MacOS/xbin/docker", "HUAQ24HBR6")),
            _ => None,
        };
        let signed = cli.map(|(cli, team)| (app.path.join(cli), fresher_core::TeamId::new(team).unwrap()));
        let mut apps = FakeInstalledApps::new(vec![app]);
        if let Some((cli, team)) = signed {
            apps = apps.signing(cli, team);
        }
        Self { apps, ..Self::none() }
    }

    /// `app` installed and open.
    pub(crate) fn open(app: InstalledApp) -> Self {
        let name = app.name.clone();
        let id = app.bundle_id.as_str().to_owned();
        Self { running: FakeRunningApps::new(vec![running_app(&name, &[&id]).unwrap()]), ..Self::installed(app) }
    }

    /// `OrbStack` installed and open, its read answered with `stdout`.
    pub(crate) fn orbstack_says(stdout: &str) -> Self {
        Self { reply: Some(Reply::Says(stdout.to_owned())), ..Self::open(orbstack()) }
    }

    pub(crate) fn reply(self, reply: Reply) -> Self {
        Self { reply: Some(reply), ..self }
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

/// What a scan did: the module's run and every command it ran, in order, and the only read it may make.
pub(crate) struct Scan {
    pub(crate) run: ModuleRun,
    pub(crate) calls: Vec<String>,
    pub(crate) read: String,
}

/// The home `name` scanned with `setup`, the clock at `FIXTURE_NOW`.
pub(crate) fn scan(name: &str, setup: Setup) -> Scan {
    let tree = home(name);
    let read = orbstack_read(tree.root());
    let commands = match setup.reply {
        None => FakeCommandRunner::default(),
        Some(Reply::Says(stdout)) => FakeCommandRunner::default().answering(&read, &stdout),
        Some(Reply::Exits(status)) => FakeCommandRunner::default().exiting(&read, status),
    };
    let ports = Ports {
        apps: Arc::new(setup.apps),
        running: Arc::new(setup.running),
        commands,
        restored: setup.restored,
        avoid: setup.avoid,
        ..Ports::at(FIXTURE_NOW)
    };
    let run = run_module(&DockerModule::new().unwrap(), &tree, &ports).unwrap();
    Scan { run, calls: ports.commands.calls(), read }
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
    scan("docker-answered", Setup::open(orbstack()).reply(Reply::Exits(1)))
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
