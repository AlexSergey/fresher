use std::path::PathBuf;

use fresher_core::{ByteSize, DiskSpace, VolumeFacts, VolumeId};
use fresher_testing::FakeVolumes;

fn facts(id: u128) -> VolumeFacts {
    VolumeFacts { id: VolumeId::new(id), device: 1, case_sensitive: false, used: ByteSize::new(1) }
}

fn disk(name: &str) -> DiskSpace {
    FakeVolumes::disk(name, ByteSize::new(100), ByteSize::new(40), ByteSize::new(60))
}

fn paths(paths: &[&str]) -> Vec<PathBuf> {
    paths.iter().map(PathBuf::from).collect()
}

mod negative {
    use std::path::Path;

    use fresher_testing::FakeVolumes;

    use super::{disk, facts, paths};
    use crate::storage::roots::{Group, RootVolumes, expand_home};

    #[test]
    fn only_a_leading_tilde_is_the_home() {
        let roots = paths(&["/a/~", "~user/x", "/b"]);

        assert_eq!(expand_home(&roots, Some(Path::new("/home"))), roots);
        assert_eq!(expand_home(&paths(&["~"]), None), paths(&["~"]));
    }

    #[test]
    fn without_a_port_every_root_is_a_volume_without_facts() {
        let volumes = RootVolumes::read(None, None, &paths(&["/a", "/b"]));

        let groups = volumes.group(&paths(&["/a", "/b"]));

        assert_eq!(groups, [Group { space: None, roots: vec![0] }, Group { space: None, roots: vec![1] }]);
    }

    #[test]
    fn a_volume_that_cannot_be_read_has_no_facts_and_a_root_not_read_stands_alone() {
        let port = FakeVolumes::default().with("/", facts(1)).with("/x", facts(2)).with_space("/x", disk("X"));

        let volumes = RootVolumes::read(Some(&port), None, &paths(&["/a", "/x"]));
        let groups = volumes.group(&paths(&["/x", "/new", "/a"]));

        let x = disk("X");
        assert_eq!(
            groups,
            [
                Group { space: None, roots: vec![2] },
                Group { space: Some(&x), roots: vec![0] },
                Group { space: None, roots: vec![1] },
            ]
        );
    }
}

mod positive {
    use std::path::Path;

    use fresher_testing::FakeVolumes;

    use super::{disk, facts, paths};
    use crate::storage::roots::{Group, RootVolumes, expand_home};

    #[test]
    fn the_tilde_becomes_the_home() {
        let expanded = expand_home(&paths(&["~", "~/Projects", "/Volumes/Data"]), Some(Path::new("/Users/me")));

        assert_eq!(expanded, paths(&["/Users/me", "/Users/me/Projects", "/Volumes/Data"]));
    }

    #[test]
    fn roots_of_one_volume_share_it_and_the_home_comes_first() {
        let port = FakeVolumes::default()
            .with("/", facts(1))
            .with("/Volumes/Data", facts(2))
            .with_space("/", disk("Macintosh HD"))
            .with_space("/Volumes/Data", disk("Data"));
        let roots = paths(&["/Volumes/Data/a", "/Users/me", "/Volumes/Data/b"]);

        let volumes = RootVolumes::read(Some(&port), Some(Path::new("/Users/me")), &roots);
        let groups = volumes.group(&roots);

        let (home, data) = (disk("Macintosh HD"), disk("Data"));
        assert_eq!(
            groups,
            [Group { space: Some(&home), roots: vec![1] }, Group { space: Some(&data), roots: vec![0, 2] }]
        );
        assert_eq!(volumes.asked(), roots);
        assert_eq!(port.space_calls(), 2);
    }
}
