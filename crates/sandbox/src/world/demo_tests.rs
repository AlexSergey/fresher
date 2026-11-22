//! The world of a demo: the allocated sizes it states and the volume it is on.

use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use fresher_core::{ByteSize, Protection};

use crate::{CaseFile, Sandbox, World, generate};

/// A demo world of `sizes` (`sparse` or `as-declared`) and an optional `[demo.volume]` table, then `cases`.
fn demo(sizes: &str, volume: &str, cases: &str) -> CaseFile {
    CaseFile::parse(&format!(
        "feature = \"developer-mac\"\n[demo]\nnow = 1795000000\nsizes = \"{sizes}\"\nshows = [\"trial.caches\"]\n\
         {volume}{cases}"
    ))
    .unwrap()
}

const VOLUME: &str =
    "[demo.volume]\nname = \"Macintosh HD\"\ncapacity = 994662584320\nused = 701202362368\nfree = 293460221952\n";

/// A scenery case with a sparse file of 6 GiB, a data file and a sparse file owned by the system.
const SCENERY: &str = r#"
[[case]]
name = "Xcode's build products"
kind = "scenery"
[[case.file]]
path = "home/Library/Developer/Xcode/DerivedData/Atlas-bxkqzr/Atlas.bin"
size = 6442450944
[[case.file]]
path = "home/Library/Caches/com.example.Browser/data.bin"
size = 4096
fill = "data"
[[case.file]]
path = "home/Library/Logs/system.log"
size = 1048576
owner = "system"
"#;

const SPARSE: &str = "home/Library/Developer/Xcode/DerivedData/Atlas-bxkqzr/Atlas.bin";
const DATA: &str = "home/Library/Caches/com.example.Browser/data.bin";
const OWNED: &str = "home/Library/Logs/system.log";

fn of(files: &[CaseFile]) -> World {
    let now = files.iter().find_map(CaseFile::demo).unwrap().now();
    World::of(files, now).unwrap()
}

fn allocated(world: &World, path: &str) -> Option<u64> {
    world
        .stated(Path::new("/box"))
        .get(&Path::new("/box").join(path))
        .and_then(|stated| stated.allocated)
        .map(ByteSize::bytes)
}

mod negative {
    use std::time::{Duration, SystemTime};

    use super::{DATA, OWNED, SCENERY, SPARSE, VOLUME, allocated, demo, of};
    use crate::{CaseFile, World};

    #[test]
    fn a_demo_world_of_sparse_sizes_states_no_allocated_size() {
        let world = of(&[demo("sparse", "", SCENERY)]);

        for path in [SPARSE, DATA, OWNED] {
            assert_eq!(allocated(&world, path), None, "{path}");
        }
        assert!(!world.text().unwrap().contains("allocated ="));
    }

    #[test]
    fn a_case_file_that_is_no_demo_states_no_allocated_size() {
        let file = CaseFile::parse(
            "feature = \"trial\"\n[[case]]\nname = \"a sparse cache\"\nkind = \"negative\"\n\
             [[case.file]]\npath = \"home/Library/Caches/big.bin\"\nsize = 1048576\n",
        )
        .unwrap();

        let world = World::of(&[file], SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000)).unwrap();

        assert_eq!(allocated(&world, "home/Library/Caches/big.bin"), None);
        assert_eq!(world.volume(), None);
    }

    #[test]
    fn two_demo_worlds_stating_different_volumes_are_refused() {
        let other = VOLUME.replace("Macintosh HD", "Data");
        let files = [demo("as-declared", VOLUME, SCENERY), demo("as-declared", &other, SCENERY)];
        let now = files[0].demo().unwrap().now();

        let error = World::of(&files, now).unwrap_err();

        let text = error.to_string();
        assert!(text.contains("Macintosh HD") && text.contains("Data"), "{text}");
    }

    #[test]
    fn a_world_without_a_volume_writes_none() {
        let text = of(&[demo("as-declared", "", SCENERY)]).text().unwrap();

        assert!(!text.contains("[volume]"), "{text}");
    }

    #[test]
    fn an_unknown_key_or_an_impossible_volume_in_world_toml_is_refused() {
        let text = of(&[demo("sparse", VOLUME, SCENERY)]).text().unwrap();

        let refusals = [
            ("free = ", "purgeable = 1\nfree = ", "purgeable"),
            ("used = 701202362368", "used = 994662584320", "[volume]: `volume.used`"),
        ];
        for (from, to, named) in refusals {
            let error = World::parse(&text.replace(from, to)).unwrap_err();

            assert!(error.contains(named), "{error}");
        }
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::ByteSize;
    use fresher_testing::StatedOwner;

    use super::{DATA, OWNED, SCENERY, SPARSE, VOLUME, allocated, demo, generated, of};
    use crate::{DemoVolume, World};

    #[test]
    fn a_demo_world_as_declared_states_each_sparse_files_size_and_no_data_files() {
        let world = of(&[demo("as-declared", "", SCENERY)]);

        assert_eq!(allocated(&world, SPARSE), Some(6_442_450_944));
        assert_eq!(allocated(&world, OWNED), Some(1_048_576));
        assert_eq!(allocated(&world, DATA), None);
    }

    #[test]
    fn an_owner_and_an_allocated_size_of_one_path_are_one_entry() {
        let world = of(&[demo("as-declared", "", SCENERY)]);

        let stated = world.stated(Path::new("/box"));
        let owned = stated.get(&Path::new("/box").join(OWNED)).unwrap();
        assert_eq!((owned.owner, owned.allocated), (Some(StatedOwner::System), Some(ByteSize::new(1_048_576))));
        assert_eq!(stated.len(), 2);
        let text = world.text().unwrap();
        assert_eq!(text.lines().filter(|line| *line == "[[entry]]").count(), 2, "{text}");
        assert_eq!(World::parse(&text).unwrap(), world);
    }

    #[test]
    fn two_demo_worlds_stating_the_same_volume_are_one_volume() {
        let files = [demo("as-declared", VOLUME, SCENERY), demo("sparse", VOLUME, "")];
        let now = files[0].demo().unwrap().now();

        let world = World::of(&files, now).unwrap();

        assert_eq!(world.volume().map(DemoVolume::capacity), Some(994_662_584_320));
    }

    #[test]
    fn the_volume_round_trips_through_world_toml() {
        let file = demo("as-declared", VOLUME, SCENERY);
        let expected = file.demo().unwrap().volume().cloned();
        let (_folder, sandbox) = generated(file);

        let world = World::read(&sandbox).unwrap();

        assert_eq!(world.volume().cloned(), expected);
        assert_eq!(
            world.volume().map(|volume| (volume.name(), volume.capacity())),
            Some(("Macintosh HD", 994_662_584_320))
        );
        assert_eq!(allocated(&world, SPARSE), Some(6_442_450_944));
        assert!(std::fs::read_to_string(sandbox.world()).unwrap().contains("[volume]\nname = \"Macintosh HD\""));
    }
}

/// `file` generated at its own `now` into a temporary folder, opened.
fn generated(file: CaseFile) -> (tempfile::TempDir, Sandbox) {
    let folder = tempfile::tempdir().unwrap();
    let user = std::fs::metadata(folder.path()).unwrap().uid();
    let now = file.demo().unwrap().now();
    generate(&[file], folder.path(), now, &Protection::default(), user).unwrap();
    let sandbox = Sandbox::open(folder.path(), user).unwrap();
    (folder, sandbox)
}
