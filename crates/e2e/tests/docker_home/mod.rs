//! What the docker flows share: a fixture sandbox (its marker at the top, the home `home/`) scanned by the real
//! scanner, its reader seen through the facts lens, with `dev-caches` and `docker` registered in the app's order; the
//! real settings file, scan cache and journal in the sandbox's data folder; the real `DiskCleaner` reaching only inside
//! the sandbox folder, with a fake Trash at `home/.Trash` (the shared flow home of `support/home*.rs`). `OrbStack` is
//! installed; whether it is open and what the command runner answers are what a flow sets, and the flow keeps the
//! runner to read every call made. `restart` builds a new engine on the same files, as when the app starts again.

#[path = "../support/events.rs"]
mod events;
#[path = "../support/home.rs"]
#[expect(dead_code, reason = "the flows of other modules restart with their own parts")]
mod home;
#[expect(dead_code, reason = "the flows of other modules read the outcomes and the journal it also gives")]
#[path = "../support/home_cleanup.rs"]
mod home_cleanup;
#[path = "../support/home_engine.rs"]
mod home_engine;
#[path = "../support/home_scan.rs"]
mod home_scan;
#[expect(dead_code, reason = "the flows of other modules sort and unnumber their reports")]
#[path = "../support/report.rs"]
mod report;

use std::path::Path;
use std::sync::Arc;

use fresher_core::{Event, Insight, InstalledApp, ModuleId, ModuleOutcome};
use fresher_module_dev_caches::DevCachesModule;
use fresher_module_docker::DockerModule;
use fresher_testing::{
    FIXTURE_NOW, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeInstalledApps, FakeRunningApps, FixtureTree,
    installed_app, running_app,
};

use crate::support::fixtures;
pub(crate) use home::Flow;
use home_engine::{Extras, Parts, engine_with, no_installs, unknown_access};

/// The module's id.
pub(crate) const MODULE: &str = "docker";
/// `OrbStack`'s socket and disk file, as the fixture makes them.
pub(crate) const SOCKET: &str = "home/.orbstack/run/docker.sock";
pub(crate) const DISK: &str = "home/Library/Group Containers/HUAQ24HBR6.dev.orbstack/data/data.img";
/// npm's cache, `dev-caches`' to offer.
pub(crate) const NPM: (&str, &str) = ("dev-caches.npm", "home/.npm/_cacache");

/// The design's answer: something unused in every kind but the stopped containers.
pub(crate) const ANSWER: &str = r#"{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}
"#;

/// `OrbStack`, installed in `/Applications`.
pub(crate) fn orbstack() -> InstalledApp {
    installed_app("dev.kdrag0n.MacVirt", "OrbStack", "2.2.5").unwrap()
}

/// The only call the module may make in the sandbox whose home is `home`, as `FakeCommandRunner` keys it: `OrbStack`'s
/// own CLI inside its bundle, through the home's socket, with the system's empty folder as its config.
pub(crate) fn read(home: &Path) -> String {
    let program = orbstack().path.join("Contents/MacOS/xbin/docker");
    let socket = home.join(".orbstack/run/docker.sock");
    format!(
        "{} --host unix://{} --config /private/var/empty system df --format json",
        program.display(),
        socket.display()
    )
}

/// How the read answers, if it is made.
enum Reply {
    Nothing,
    Says(&'static str),
    Cancelled,
}

/// The world an engine runs in besides the files.
pub(crate) struct Setup {
    pub(crate) running: FakeRunningApps,
    reply: Reply,
}

impl Setup {
    /// `OrbStack` open, its read answered with the design's answer.
    pub(crate) fn answered() -> Self {
        Self { running: open(), reply: Reply::Says(ANSWER) }
    }

    /// `OrbStack` closed, no command answered.
    pub(crate) fn closed() -> Self {
        Self { running: FakeRunningApps::default(), reply: Reply::Nothing }
    }

    /// `OrbStack` open, its read stopped by a cancel.
    pub(crate) fn cancelled() -> Self {
        Self { running: open(), reply: Reply::Cancelled }
    }

    /// The runner of this setup in the sandbox whose home is `home`.
    fn commands(&self, home: &Path) -> FakeCommandRunner {
        match self.reply {
            Reply::Nothing => FakeCommandRunner::default(),
            Reply::Says(stdout) => FakeCommandRunner::default().answering(&read(home), stdout),
            Reply::Cancelled => {
                let cancelled = fresher_core::CommandRunError::Cancelled { program: "docker".into() };
                FakeCommandRunner::default().failing(&read(home), cancelled)
            }
        }
    }
}

fn open() -> FakeRunningApps {
    FakeRunningApps::new(vec![running_app("OrbStack", &["dev.kdrag0n.MacVirt"]).unwrap()])
}

/// The flow home with the command runner its engine was given.
pub(crate) struct Home {
    pub(crate) flow: Flow,
    pub(crate) commands: Arc<FakeCommandRunner>,
    /// The only call the module may make in this sandbox.
    pub(crate) read: String,
}

impl Home {
    /// The sandbox holding `fixtures/homes/docker-flow.toml`, its engine in `setup`.
    pub(crate) fn with(setup: Setup) -> Self {
        let tree = FixtureTree::from_toml(&fixtures().join("homes").join("docker-flow.toml")).unwrap();
        let mut flow = Flow::open(tree, |_| parts(FakeRunningApps::default()));
        let commands = Arc::new(setup.commands(&flow.sandbox.home()));
        let read = read(&flow.sandbox.home());
        // `Flow::open` builds the engine of the defaults; the flow's own takes its place before anything runs.
        flow.engine =
            engine_with(&flow.sandbox, &flow.stated, &flow.open_files, parts(setup.running), extras(&commands));
        Self { flow, commands, read }
    }

    /// A new engine on the same files in `setup`: the app started again. Its runner takes the old one's place.
    pub(crate) fn restart(&mut self, setup: Setup) {
        self.commands = Arc::new(setup.commands(&self.flow.sandbox.home()));
        let parts = parts(setup.running);
        self.flow.engine =
            engine_with(&self.flow.sandbox, &self.flow.stated, &self.flow.open_files, parts, extras(&self.commands));
    }
}

fn extras(commands: &Arc<FakeCommandRunner>) -> Extras {
    Extras { commands: Arc::clone(commands), ..Extras::default() }
}

/// The module's `ModuleFinished` in `events`: its outcome and its insights.
pub(crate) fn finished(events: &[Event]) -> (ModuleOutcome, Vec<Insight>) {
    let module = ModuleId::new(MODULE).unwrap();
    events
        .iter()
        .find_map(|event| match event {
            Event::ModuleFinished { module: finished, outcome, insights, .. } if *finished == module => {
                Some((outcome.clone(), insights.clone()))
            }
            _ => None,
        })
        .unwrap()
}

/// The outcome of module `id` in `events`.
pub(crate) fn outcome(events: &[Event], id: &'static str) -> ModuleOutcome {
    let module = ModuleId::new(id).unwrap();
    events
        .iter()
        .find_map(|event| match event {
            Event::ModuleFinished { module: finished, outcome, .. } if *finished == module => Some(outcome.clone()),
            _ => None,
        })
        .unwrap()
}

/// The engine's parts with `running` open: the app's modules of this home in its order, `OrbStack` installed, two
/// threads.
fn parts(running: FakeRunningApps) -> Parts {
    Parts {
        modules: vec![Box::new(DevCachesModule::new().unwrap()), Box::new(DockerModule::new().unwrap())],
        clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
        journal: FakeChangeJournal::default(),
        // OrbStack's own CLI carries OrbStack's signature: the module runs no other.
        installed: FakeInstalledApps::new(vec![orbstack()]).signing(
            orbstack().path.join("Contents/MacOS/xbin/docker"),
            fresher_core::TeamId::new("HUAQ24HBR6").unwrap(),
        ),
        running,
        access: unknown_access(),
        installs: no_installs(),
        threads: 2.try_into().unwrap(),
        avoid: Vec::new(),
    }
}
