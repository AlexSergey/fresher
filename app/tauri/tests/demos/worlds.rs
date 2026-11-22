//! One demo world scanned: generated into a temporary folder at its own `now`, scanned with every module, and checked
//! against its `shows`, its sandbox and its volume.

use std::fs;

use fresher_sandbox::CaseFile;

use crate::harness::{Failure, World, failure, findings, fixtures, path_of, scan};

/// The names of the demo worlds in `fixtures/demos/`, without `.toml`, sorted.
pub(crate) fn demo_world_names() -> Vec<String> {
    let entries = fs::read_dir(fixtures().join("demos")).unwrap();
    let mut names: Vec<String> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The scan test of `fixtures/demos/<name>.toml`: it passes, or fails naming the world and what is wrong.
pub(crate) fn check_demo_world(name: &str) {
    let path = fixtures().join("demos").join(format!("{name}.toml"));
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display())).unwrap();
    check(name, &text).map_err(|failure| format!("demo world {name}: {}", failure.problem)).unwrap();
}

/// Scans the demo world `text` and checks that every category it `shows` offers something, that nothing is offered
/// outside the sandbox and that the storage map is on the world's volume.
pub(crate) fn check(name: &str, text: &str) -> Result<(), Failure> {
    let file = CaseFile::parse(text).map_err(|error| failure(name, format!("the world is refused: {error}")))?;
    let demo = file.demo().ok_or_else(|| failure(name, "the file has no [demo] header"))?;
    let world = World::new(name)?;
    let (engine, _) = world.engine(std::slice::from_ref(&file), None)?;
    scan(name, &engine)?;
    let sandbox = world.sandbox();
    let findings = findings(name, &engine)?;

    for category in demo.shows() {
        if !findings.iter().any(|found| found.finding().category() == *category) {
            let mut offered: Vec<&str> = findings.iter().map(|found| found.finding().category().as_str()).collect();
            offered.sort_unstable();
            offered.dedup();
            let problem =
                format!("the category `{}` offers nothing; offered: {}", category.as_str(), offered.join(", "));
            return Err(failure(name, problem));
        }
    }
    // The app's own `demo` module (debug builds) offers made-up paths of its own; it is no part of any world.
    let of_the_world = findings.iter().filter(|found| found.finding().category().module().as_str() != "demo");
    if let Some(outside) = of_the_world.filter_map(path_of).find(|found| !found.starts_with(&sandbox)) {
        return Err(failure(name, format!("a finding outside the sandbox: {}", outside.display())));
    }
    check_volume(name, demo.volume().map(fresher_sandbox::DemoVolume::capacity), &engine)
}

/// The storage map's volume has the world's capacity (when the world states a volume).
fn check_volume(name: &str, stated: Option<u64>, engine: &fresher_engine::Engine) -> Result<(), Failure> {
    let Some(capacity) = stated else { return Ok(()) };
    let map = engine.storage_map().map_err(|error| failure(name, format!("the map: {error:?}")))?;
    let volume = map.volumes.first().ok_or_else(|| failure(name, "the storage map holds no volume"))?;
    if volume.capacity().bytes() != capacity {
        let problem = format!("the map's volume holds {} bytes, the world's is {capacity}", volume.capacity().bytes());
        return Err(failure(name, problem));
    }
    Ok(())
}
