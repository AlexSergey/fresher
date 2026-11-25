//! One demo world scanned: generated into a temporary folder at its own `now`, scanned with every module, and checked
//! against its `shows`, its sandbox and its volume.

use std::fs;

use fresher_core::{ByteSize, Completeness, PartKind};
use fresher_sandbox::{CaseFile, DemoVolume};

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
    if let Some(outside) = findings.iter().filter_map(path_of).find(|found| !found.starts_with(&sandbox)) {
        return Err(failure(name, format!("a finding outside the sandbox: {}", outside.display())));
    }
    check_complete(name, &engine)?;
    check_volume(name, demo.volume(), &engine)
}

/// A demo shows a Mac whose scan saw everything: every guarded place entered, every folder read, and every size whole,
/// so no notice of folders not scanned and no `+` after a size is in the picture.
fn check_complete(name: &str, engine: &fresher_engine::Engine) -> Result<(), Failure> {
    let overview = engine.overview().map_err(|error| failure(name, format!("the overview: {error:?}")))?;
    let coverage = overview.last_scan.map(|last| last.coverage);
    if !coverage.as_ref().is_some_and(fresher_core::Coverage::is_complete) {
        return Err(failure(name, format!("the scan did not see everything: {coverage:?}")));
    }
    if overview.found.completeness != Completeness::Complete {
        return Err(failure(name, "the size of what can be freed is not whole"));
    }
    let map = engine.storage_map().map_err(|error| failure(name, format!("the map: {error:?}")))?;
    let parts = map.volumes.iter().flat_map(fresher_core::VolumeMap::parts);
    if let Some(part) = parts.into_iter().find(|part| part.sizes.completeness != Completeness::Complete) {
        return Err(failure(name, format!("the map's part {:?} is not whole", part.kind)));
    }
    Ok(())
}

/// The storage map's volume has the world's capacity and macOS's stated size (when the world states a volume).
fn check_volume(name: &str, stated: Option<&DemoVolume>, engine: &fresher_engine::Engine) -> Result<(), Failure> {
    let Some(stated) = stated else { return Ok(()) };
    let map = engine.storage_map().map_err(|error| failure(name, format!("the map: {error:?}")))?;
    let volume = map.volumes.first().ok_or_else(|| failure(name, "the storage map holds no volume"))?;
    if volume.capacity().bytes() != stated.capacity() {
        let problem =
            format!("the map's volume holds {} bytes, the world's is {}", volume.capacity().bytes(), stated.capacity());
        return Err(failure(name, problem));
    }
    let macos = volume.parts().iter().find(|part| part.kind == PartKind::MacOs).map(|part| part.sizes.allocated);
    if macos.map(ByteSize::bytes) != Some(stated.macos()) {
        return Err(failure(name, format!("the map's macOS part is {macos:?}, the world's is {}", stated.macos())));
    }
    Ok(())
}
