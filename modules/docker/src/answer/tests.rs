//! The answer's format: every unreadable shape, then what reads.

/// The four lines of Docker's documentation (the design's section 3).
const DOCUMENTED: &str = r#"{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}
"#;

/// A line of `kind` with the given counts and sizes.
fn line(kind: &str, total: &str, active: &str, size: &str, reclaimable: &str) -> String {
    format!(
        r#"{{"Type":"{kind}","TotalCount":"{total}","Active":"{active}","Size":"{size}","Reclaimable":"{reclaimable}"}}"#
    )
}

/// The documented answer with the line of `kind` replaced by `replacement`.
fn replacing(kind: &str, replacement: &str) -> String {
    DOCUMENTED
        .lines()
        .map(
            |text| if text.contains(&format!(r#""Type":"{kind}""#)) { replacement.to_owned() } else { text.to_owned() },
        )
        .collect::<Vec<_>>()
        .join("\n")
}

mod negative {
    use super::{DOCUMENTED, line, replacing};
    use crate::answer::{parse, size};

    #[test]
    fn an_empty_answer_is_unreadable() {
        assert!(parse("").is_err());
        assert!(parse("\n  \n").is_err());
    }

    #[test]
    fn a_line_that_is_not_json_is_unreadable() {
        assert!(parse(&format!("{DOCUMENTED}TYPE SIZE\n")).is_err());
        assert!(parse(&format!("{DOCUMENTED}[1, 2]\n")).is_err());
        assert!(parse(&replacing("Images", r#"{"Type":"Images","TotalCount":7}"#)).is_err());
    }

    #[test]
    fn a_missing_kind_is_unreadable() {
        let without_volumes: String =
            DOCUMENTED.lines().filter(|text| !text.contains("Local Volumes")).collect::<Vec<_>>().join("\n");

        assert!(parse(&without_volumes).is_err());
        assert!(
            parse(&replacing("Images", r#"{"Type":"Images","TotalCount":"7","Active":"2","Size":"3GB"}"#)).is_err()
        );
    }

    #[test]
    fn a_kind_twice_is_unreadable() {
        let twice = format!("{DOCUMENTED}{}\n", line("Containers", "1", "1", "1kB", "0B"));

        assert!(parse(&twice).is_err());
    }

    #[test]
    fn a_size_without_a_unit_is_unreadable() {
        assert!(parse(&replacing("Build Cache", &line("Build Cache", "1", "0", "512", "512MB"))).is_err());
        for text in ["512", "MB", ".5MB", "5.MB", "1 MB", "-1MB", ""] {
            assert!(size(text).is_err(), "{text}");
        }
    }

    #[test]
    fn an_unknown_unit_is_unreadable() {
        for unit in ["GiB", "XB", "mb", "b"] {
            assert!(size(&format!("1{unit}")).is_err(), "{unit}");
        }
        assert!(parse(&replacing("Images", &line("Images", "7", "2", "3GiB", "1GB"))).is_err());
    }

    #[test]
    fn more_active_than_total_is_unreadable() {
        assert!(parse(&replacing("Images", &line("Images", "2", "3", "3GB", "1GB"))).is_err());
    }

    #[test]
    fn a_count_that_is_not_a_whole_number_is_unreadable() {
        for count in ["+2", "-1", "2.0", "", "two", "99999999999999999999999"] {
            assert!(parse(&replacing("Images", &line("Images", count, "0", "3GB", "1GB"))).is_err(), "{count}");
        }
    }

    #[test]
    fn an_answer_longer_than_64_kib_is_unreadable() {
        let padded = format!("{DOCUMENTED}{}", " \n".repeat(64 * 1024));

        assert!(parse(&padded).is_err());
    }

    #[test]
    fn more_unused_than_the_size_is_unreadable() {
        assert!(parse(&replacing("Images", &line("Images", "7", "2", "3GB", "3.1GB (103%)"))).is_err());
        assert!(parse(&replacing("Images", &line("Images", "7", "2", "3GB", "1GB (x%)"))).is_err());
    }
}

mod positive {
    use fresher_core::ByteSize;
    use proptest::prelude::*;

    use super::{DOCUMENTED, line, replacing};
    use crate::answer::{Kind, Usage, parse, size};

    #[test]
    fn the_documented_answer_reads() {
        let kind = |size, unused, count, active| Kind {
            size: ByteSize::new(size),
            unused: ByteSize::new(unused),
            count,
            active,
        };

        assert_eq!(
            parse(DOCUMENTED),
            Ok(Usage {
                build_cache: kind(512_000_000, 512_000_000, 13, 0),
                images: kind(3_034_000_000, 1_071_000_000, 7, 2),
                containers: kind(1090, 0, 1, 1),
                volumes: kind(41_200_000, 41_200_000, 2, 1),
            })
        );
        assert_eq!(parse(DOCUMENTED).unwrap().images.unused_count(), 5);
    }

    #[test]
    fn units_are_powers_of_1000() {
        let read = |text: &str| size(text).unwrap().bytes();

        assert_eq!(read("7B"), 7);
        assert_eq!(read("1kB"), 1_000);
        assert_eq!(read("1KB"), 1_000);
        assert_eq!(read("1MB"), 1_000_000);
        assert_eq!(read("1GB"), 1_000_000_000);
        assert_eq!(read("1TB"), 1_000_000_000_000);
        assert_eq!(read("1PB"), 1_000_000_000_000_000);
        assert_eq!(read("1.5kB"), 1_500);
        assert_eq!(read("0.0005kB"), 1);
    }

    #[test]
    fn a_percentage_after_reclaimable_is_dropped() {
        let answer = replacing("Images", &line("Images", "7", "2", "3GB", "1.5GB (50.0%)"));

        assert_eq!(parse(&answer).unwrap().images.unused, ByteSize::new(1_500_000_000));
    }

    #[test]
    fn an_unknown_kind_is_ignored() {
        let answer =
            format!("{DOCUMENTED}{{\"Type\":\"Snapshots\",\"Size\":\"many\"}}\n{{\"Type\":\"X\",\"Size\":5}}\n");

        assert_eq!(parse(&answer), parse(DOCUMENTED));
    }

    #[test]
    fn unknown_fields_are_ignored() {
        let answer = DOCUMENTED.replace(r#""Type":"Images""#, r#""Type":"Images","Shared":"9GB","Extra":1"#);

        assert_eq!(parse(&answer), parse(DOCUMENTED));
    }

    proptest! {
        #[test]
        fn a_size_printed_in_docker_s_units_reads_back_within_rounding(
            bytes in 0_u64..=10_u64.pow(16),
            unit in 0_usize..7,
            decimals in 0_u32..=3,
        ) {
            let (name, multiplier) = [
                ("B", 1_u128),
                ("kB", 1_000),
                ("KB", 1_000),
                ("MB", 1_000_000),
                ("GB", 1_000_000_000),
                ("TB", 1_000_000_000_000),
                ("PB", 1_000_000_000_000_000),
            ][unit];
            let scale = 10_u128.pow(decimals);
            let shown = (u128::from(bytes) * scale + multiplier / 2) / multiplier;
            let digits = format!("{shown:0>width$}", width = decimals as usize + 1);
            let (whole, fraction) = digits.split_at(digits.len() - decimals as usize);
            let printed = if decimals == 0 { format!("{whole}{name}") } else { format!("{whole}.{fraction}{name}") };

            let read = u128::from(size(&printed).unwrap().bytes());

            prop_assert!(read.abs_diff(u128::from(bytes)) <= multiplier / scale / 2 + 1, "{printed} → {read}");
        }
    }
}
