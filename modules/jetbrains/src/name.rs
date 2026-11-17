//! Reading a version folder's name (`WebStorm2025.3`) and an installed app's version string (`2025.3.1`).

use std::ffi::OsStr;
use std::fmt;

use crate::products::{Product, Products};

/// The first and last year a version may name: four digits starting `20`.
const YEARS: std::ops::RangeInclusive<u16> = 2000..=2099;
/// The minors a version may name: one or two digits with no leading zero.
const MINORS: std::ops::RangeInclusive<u8> = 1..=99;

/// An IDE's version as its folders name it: `2025.3`. Ordered by year, then minor (`2025.10` after `2025.9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Version {
    year: u16,
    minor: u8,
}

impl Version {
    /// A version, if `year` has four digits starting `20` and `minor` is between 1 and 99.
    pub(crate) fn new(year: u16, minor: u8) -> Option<Self> {
        (YEARS.contains(&year) && MINORS.contains(&minor)).then_some(Self { year, minor })
    }

    #[cfg(test)]
    pub(crate) const fn year(self) -> u16 {
        self.year
    }

    #[cfg(test)]
    pub(crate) const fn minor(self) -> u8 {
        self.minor
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.year, self.minor)
    }
}

/// The product and version a folder's name names: exactly a product's folder name, case included, then
/// `<year>.<minor>` and nothing after. `None` for anything else, so the folder is never claimed.
pub(crate) fn parse<'a>(name: &OsStr, products: &'a Products) -> Option<(&'a Product, Version)> {
    let name = name.to_str()?;
    let split = name.find(|char: char| !char.is_ascii_alphabetic())?;
    let (folder, version) = name.split_at(split);
    let product = products.by_folder(folder)?;
    let (version, rest) = leading_version(version)?;
    rest.is_empty().then_some((product, version))
}

/// The version an installed app's version string starts with (`2025.3.1` and `2026.1 EAP` are 2025.3 and 2026.1);
/// `None` when it does not start with one (`253.1234`, `v2025.3`, empty): an unknown version. What follows the
/// version never starts with a digit: the minor took them all.
pub(crate) fn installed_version(text: &str) -> Option<Version> {
    leading_version(text).map(|(version, _)| version)
}

/// The `<year>.<minor>` at the start of `text` and what follows it; the minor takes every digit up to the next
/// non-digit, so `2025.123` names no version.
fn leading_version(text: &str) -> Option<(Version, &str)> {
    let (year, rest) = leading_digits(text);
    let rest = rest.strip_prefix('.')?;
    let (minor, rest) = leading_digits(rest);
    if year.len() != 4 || minor.is_empty() || minor.len() > 2 || minor.starts_with('0') {
        return None;
    }
    Some((Version::new(year.parse().ok()?, minor.parse().ok()?)?, rest))
}

/// The ASCII digits at the start of `text`, and the rest.
fn leading_digits(text: &str) -> (&str, &str) {
    let end = text.find(|char: char| !char.is_ascii_digit()).unwrap_or(text.len());
    text.split_at(end)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use crate::name::{Version, parse};
    use crate::products::Products;

    /// The folder and version `name` parses to with the shipped table, if any.
    fn parsed(name: &str) -> Option<(String, String)> {
        let products = Products::shipped().unwrap();
        parse(OsStr::new(name), &products).map(|(product, version)| (product.folder().into(), version.to_string()))
    }

    fn version(year: u16, minor: u8) -> Version {
        Version::new(year, minor).unwrap()
    }

    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        use super::parsed;
        use crate::name::{Version, installed_version, parse};
        use crate::products::Products;

        #[test]
        fn refuses_the_names_the_design_lists() {
            for name in [
                "WebStorm2024",
                "WebStorm2024.3.1",
                "WebStorm2024.3.bak",
                "WebStorm2024.3 copy",
                "WebStorm-2024.3",
                "WebStorm.2024.3",
                "WebStorm2024.03",
                "WebStorm1999.1",
                "FooStorm2024.1",
                "JetBrainsClient2025.1",
            ] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_the_vendor_folders_other_names() {
            for name in ["Toolbox", "consentOptions", "PrivacyPolicy", "bl", "crl", "Gateway2025.1"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_an_empty_name_and_a_bare_product_or_version() {
            for name in ["", "WebStorm", "2024.3", ".2024.3"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_another_case_of_the_product() {
            for name in ["webstorm2024.2", "WEBSTORM2024.2", "Webstorm2024.2", "pycharmCE2024.1"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_a_year_of_three_or_five_digits() {
            for name in ["WebStorm202.1", "WebStorm20241.1", "WebStorm02024.1"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_a_minor_of_zero_or_with_a_leading_zero() {
            for name in ["WebStorm2024.0", "WebStorm2024.00", "WebStorm2024.03", "WebStorm2024.", "WebStorm2024.100"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_text_before_or_after_the_name() {
            for name in [" WebStorm2024.1", "WebStorm2024.1 ", "XWebStorm2024.1", "WebStorm2024.1a", "WebStorm2024.1/"]
            {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_a_prefix_of_one_product_followed_by_another_products_name() {
            for name in ["PyCharmWebStorm2024.1", "WebStormPyCharm2024.1", "PyCharmIdeaIC2024.1", "IdeaICCE2024.1"] {
                assert_eq!(parsed(name), None, "{name}");
            }
        }

        #[test]
        fn refuses_a_name_that_is_not_utf8() {
            let products = Products::shipped().unwrap();
            let name = OsStr::from_bytes(b"WebStorm2024.1\xff");

            assert_eq!(parse(name, &products), None);
            assert_eq!(parse(OsStr::from_bytes(b"\xffWebStorm2024.1"), &products), None);
        }

        #[test]
        fn refuses_non_ascii_digits() {
            assert_eq!(parsed("WebStorm２０２４.1"), None);
            assert_eq!(installed_version("２０２５.3"), None);
        }

        #[test]
        fn an_installed_version_that_does_not_start_with_a_version_is_unknown() {
            for text in
                ["253.1234", "", "v2025.3", " 2025.3", "2025", "2025.", "2025.03", "2025.0", "2025.123", "1999.1"]
            {
                assert_eq!(installed_version(text), None, "{text}");
            }
        }

        #[test]
        fn a_version_outside_the_years_and_minors_is_refused() {
            assert_eq!(Version::new(1999, 1), None);
            assert_eq!(Version::new(2100, 1), None);
            assert_eq!(Version::new(2025, 0), None);
            assert_eq!(Version::new(2025, 100), None);
        }
    }

    mod positive {
        use super::{parsed, version};
        use crate::name::installed_version;
        use crate::products::Products;

        #[test]
        fn reads_each_products_folder_name() {
            let products = Products::shipped().unwrap();

            for product in products.all() {
                let name = format!("{}2024.3", product.folder());

                assert_eq!(parsed(&name), Some((product.folder().into(), "2024.3".into())), "{name}");
            }
        }

        #[test]
        fn the_community_editions_are_their_own_products() {
            assert_eq!(parsed("PyCharmCE2024.1"), Some(("PyCharmCE".into(), "2024.1".into())));
            assert_eq!(parsed("PyCharm2024.1"), Some(("PyCharm".into(), "2024.1".into())));
            assert_eq!(parsed("IdeaIC2024.1"), Some(("IdeaIC".into(), "2024.1".into())));
            assert_eq!(parsed("IntelliJIdea2024.1"), Some(("IntelliJIdea".into(), "2024.1".into())));
        }

        #[test]
        fn reads_a_two_digit_minor() {
            assert_eq!(parsed("WebStorm2025.10"), Some(("WebStorm".into(), "2025.10".into())));
        }

        #[test]
        fn a_minor_of_ten_comes_after_nine_and_the_year_comes_first() {
            assert!(version(2025, 10) > version(2025, 9));
            assert!(version(2026, 1) > version(2025, 10));
            assert!(version(2024, 3) < version(2025, 1));
        }

        #[test]
        fn reads_an_installed_versions_start() {
            assert_eq!(installed_version("2025.3.1"), Some(version(2025, 3)));
            assert_eq!(installed_version("2025.3"), Some(version(2025, 3)));
            assert_eq!(installed_version("2026.1 EAP"), Some(version(2026, 1)));
            assert_eq!(installed_version("2025.10"), Some(version(2025, 10)));
        }
    }

    mod property {
        use std::ffi::OsStr;

        use proptest::prelude::*;

        use crate::name::{Version, installed_version, parse};
        use crate::products::Products;

        fn any_version() -> impl Strategy<Value = Version> {
            (2000u16..=2099, 1u8..=99).prop_map(|(year, minor)| Version::new(year, minor).unwrap())
        }

        proptest! {
            #[test]
            fn the_order_is_the_order_of_year_then_minor(a in any_version(), b in any_version()) {
                prop_assert_eq!(a.cmp(&b), (a.year(), a.minor()).cmp(&(b.year(), b.minor())));
            }

            #[test]
            fn parsing_a_built_folder_name_gives_back_its_product_and_version(
                index in any::<prop::sample::Index>(),
                version in any_version(),
            ) {
                let products = Products::shipped().unwrap();
                let product = index.get(products.all());
                let name = format!("{}{version}", product.folder());

                let parsed = parse(OsStr::new(&name), &products);

                prop_assert_eq!(parsed, Some((product, version)));
            }

            #[test]
            fn an_installed_version_reads_what_display_writes(version in any_version(), suffix in "( EAP|\\.[0-9]{1,3})?") {
                prop_assert_eq!(installed_version(&format!("{version}{suffix}")), Some(version));
            }
        }
    }
}
