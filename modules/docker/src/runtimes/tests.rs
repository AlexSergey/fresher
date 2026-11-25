//! The runtimes table: every refusal by a table made for it, then the shipped table.

use super::{Runtimes, RuntimesError};

/// One row with every field valid, `change` replacing one line (`name = ...`) by its key's prefix.
fn row_with(id: &str, change: Option<(&str, &str)>) -> String {
    let mut lines = vec![
        ("id", format!("id = \"{id}\"")),
        ("name", "name = \"Runtime\"".to_string()),
        ("app", "app = \"com.example.Runtime\"".to_string()),
        ("team", "team = \"ABCDE12345\"".to_string()),
        ("cli", "cli = \"Contents/bin/docker\"".to_string()),
        ("context", "context = \"runtime\"".to_string()),
        ("socket", format!("socket = {{ folder = \"home\", path = \".{id}/run/docker.sock\" }}")),
        ("disk", format!("disk = {{ folder = \"containers\", path = \"{id}/disk.raw\" }}")),
        ("maximum", "maximum = \"hidden\"".to_string()),
    ];
    if let Some((key, line)) = change {
        for entry in &mut lines {
            if entry.0 == key {
                entry.1 = line.to_string();
            }
        }
    }
    let body: Vec<String> = lines.into_iter().map(|(_, line)| line).collect();
    format!("[[runtime]]\n{}\n", body.join("\n"))
}

fn refused(text: &str) -> RuntimesError {
    Runtimes::parse(text).unwrap_err()
}

mod negative {
    use super::{RuntimesError, refused, row_with};

    #[test]
    fn a_row_whose_team_is_not_a_team_id_is_refused() {
        for team in ["", "abcde12345", "ABCDE1234", "ABCDE-2345"] {
            let text = row_with("a", Some(("team", &format!("team = \"{team}\""))));

            assert_eq!(refused(&text), RuntimesError::BadTeam { id: "a".into(), team: team.into() }, "{team}");
        }
    }

    #[test]
    fn a_table_that_is_not_toml_or_has_an_unknown_field_is_refused() {
        assert!(matches!(refused("[[runtime"), RuntimesError::Format(_)));
        let text = format!("{}colour = 1\n", row_with("a", None));
        assert!(matches!(refused(&text), RuntimesError::Format(_)));
    }

    #[test]
    fn a_table_with_no_row_is_refused() {
        assert_eq!(refused(""), RuntimesError::Empty);
    }

    #[test]
    fn a_row_id_that_is_not_kebab_case_is_refused() {
        for id in ["Docker", "a.b", "", "a_b"] {
            assert_eq!(refused(&row_with(id, None)), RuntimesError::BadId { id: id.into() }, "{id}");
        }
    }

    #[test]
    fn a_row_id_listed_twice_is_refused() {
        let second = row_with("a", None).replace("\"a/disk.raw\"", "\"b/disk.raw\"").replace(".a/run", ".b/run");
        let text = format!("{}{second}", row_with("a", None));
        assert_eq!(refused(&text), RuntimesError::DuplicateId { id: "a".into() });
    }

    #[test]
    fn a_name_that_is_empty_too_long_or_holds_a_control_character_is_refused() {
        let long = format!("name = \"{}\"", "x".repeat(41));
        for line in ["name = \"\"", long.as_str(), "name = \"Two\\nlines\""] {
            assert_eq!(refused(&row_with("a", Some(("name", line)))), RuntimesError::BadName { id: "a".into() });
        }
    }

    #[test]
    fn an_app_that_is_not_a_bundle_id_is_refused() {
        let text = row_with("a", Some(("app", "app = \"not a bundle id\"")));
        assert_eq!(refused(&text), RuntimesError::BadApp { id: "a".into(), app: "not a bundle id".into() });
    }

    #[test]
    fn a_cli_that_is_empty_absolute_or_not_plain_names_is_refused() {
        for cli in ["", "/usr/bin/docker", "../docker", "Contents/*/docker", "Contents/./docker"] {
            let text = row_with("a", Some(("cli", &format!("cli = \"{cli}\""))));
            assert_eq!(refused(&text), RuntimesError::BadCli { id: "a".into(), cli: cli.into() }, "{cli}");
        }
    }

    #[test]
    fn a_context_that_is_empty_or_not_lowercase_letters_digits_and_dashes_is_refused() {
        for context in ["", "Desktop", "desk top", "desk_top"] {
            let text = row_with("a", Some(("context", &format!("context = \"{context}\""))));
            assert_eq!(refused(&text), RuntimesError::BadContext { id: "a".into(), context: context.into() });
        }
    }

    #[test]
    fn a_place_that_is_empty_absolute_starred_dotted_or_walked_is_refused() {
        for path in ["", "*", "..", "/etc/docker.sock", "a/../b", "a//b"] {
            let line = format!("socket = {{ folder = \"home\", path = \"{path}\" }}");
            let text = row_with("a", Some(("socket", &line)));
            assert_eq!(refused(&text), RuntimesError::BadPlace { id: "a".into(), path: path.into() }, "{path}");
        }
        for folder in ["user-temp", "shared-temp"] {
            let line = format!("disk = {{ folder = \"{folder}\", path = \"a/disk.raw\" }}");
            let text = row_with("a", Some(("disk", &line)));
            assert_eq!(refused(&text), RuntimesError::BadPlace { id: "a".into(), path: "a/disk.raw".into() });
        }
    }

    #[test]
    fn a_place_named_twice_is_refused() {
        let same_row = row_with("a", Some(("disk", "disk = { folder = \"home\", path = \".a/run/docker.sock\" }")));
        assert_eq!(refused(&same_row), RuntimesError::DuplicatePlace { id: "a".into() });
        let other = row_with("b", Some(("socket", "socket = { folder = \"home\", path = \".a/run/docker.sock\" }")));
        let text = format!("{}{other}", row_with("a", None));
        assert_eq!(refused(&text), RuntimesError::DuplicatePlace { id: "b".into() });
    }
}

mod positive {
    use std::path::PathBuf;

    use fresher_core::{BundleId, Claim, KnownFolder, Pattern};

    use super::super::{Maximum, Place, Runtimes, Watched};

    fn place(folder: KnownFolder, path: &str) -> Place {
        Place { folder, path: PathBuf::from(path) }
    }

    #[test]
    fn the_shipped_table_has_docker_desktop_then_orbstack() {
        let table = Runtimes::shipped().unwrap();

        assert_eq!(table.rows().len(), 2);
        let (desktop, orbstack) = (&table.rows()[0], &table.rows()[1]);
        assert_eq!(desktop.id, "docker-desktop");
        assert_eq!(desktop.name, "Docker Desktop");
        assert_eq!(desktop.app, BundleId::new("com.docker.docker").unwrap());
        assert_eq!(desktop.cli, PathBuf::from("Contents/Resources/bin/docker"));
        assert_eq!(desktop.context, "desktop-linux");
        assert_eq!(desktop.socket, place(KnownFolder::Home, ".docker/run/docker.sock"));
        assert_eq!(desktop.disk, place(KnownFolder::Containers, "com.docker.docker/Data/vms/0/data/Docker.raw"));
        assert_eq!(desktop.maximum, Maximum::Shown);
        assert_eq!(orbstack.id, "orbstack");
        assert_eq!(orbstack.name, "OrbStack");
        assert_eq!(orbstack.app, BundleId::new("dev.kdrag0n.MacVirt").unwrap());
        assert_eq!(orbstack.cli, PathBuf::from("Contents/MacOS/xbin/docker"));
        assert_eq!(orbstack.context, "orbstack");
        assert_eq!(orbstack.socket, place(KnownFolder::Home, ".orbstack/run/docker.sock"));
        assert_eq!(orbstack.disk, place(KnownFolder::GroupContainers, "HUAQ24HBR6.dev.orbstack/data/data.img"));
        assert_eq!(orbstack.maximum, Maximum::Hidden);
    }

    #[test]
    fn each_row_watches_its_socket_then_its_disk() {
        let table = Runtimes::shipped().unwrap();

        let interests = table.interests().unwrap();

        assert_eq!(interests.len(), 4);
        for (index, interest) in interests.iter().enumerate() {
            let (row, watched) = table.of_interest(index).unwrap();
            let place = if watched == Watched::Socket { &row.socket } else { &row.disk };
            let expected = Pattern::Under(place.folder, place.path.to_string_lossy().into_owned());
            assert_eq!(interest.pattern(), &expected, "{index}");
        }
        let ids_and_watched: Vec<_> =
            (0..5).map(|index| table.of_interest(index).map(|(row, watched)| (row.id.as_str(), watched))).collect();
        assert_eq!(
            ids_and_watched,
            [
                Some(("docker-desktop", Watched::Socket)),
                Some(("docker-desktop", Watched::Disk)),
                Some(("orbstack", Watched::Socket)),
                Some(("orbstack", Watched::Disk)),
                None,
            ]
        );
    }

    #[test]
    fn every_interest_is_an_item_claim() {
        let interests = Runtimes::shipped().unwrap().interests().unwrap();

        assert!(interests.iter().all(|interest| interest.claim() == Claim::Item));
    }
}
