//! The module's categories and their danger levels (design, section 3.3): code, since a category id is a
//! `&'static str`.

use fresher_core::DangerLevel;

/// A category of the module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Category {
    Dependencies,
    BuildOutput,
    PythonEnvironments,
    RustBuildCache,
}

impl Category {
    /// Every category, in the order the module lists them.
    pub(crate) const ALL: [Self; 4] =
        [Self::Dependencies, Self::BuildOutput, Self::PythonEnvironments, Self::RustBuildCache];

    /// The category's name: the start of each of its findings' identities (`dependencies/Code/shop/node_modules`).
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Dependencies => "dependencies",
            Self::BuildOutput => "build-output",
            Self::PythonEnvironments => "python-environments",
            Self::RustBuildCache => "rust-build-cache",
        }
    }

    /// The category's id in the module (`stale-projects.dependencies`).
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::Dependencies => "stale-projects.dependencies",
            Self::BuildOutput => "stale-projects.build-output",
            Self::PythonEnvironments => "stale-projects.python-environments",
            Self::RustBuildCache => "stale-projects.rust-build-cache",
        }
    }

    pub(crate) const fn danger(self) -> DangerLevel {
        match self {
            Self::PythonEnvironments => DangerLevel::Medium,
            Self::Dependencies | Self::BuildOutput | Self::RustBuildCache => DangerLevel::Low,
        }
    }

    /// The category a table row names; `rust-build-cache` is no row's, since a cache is the code's.
    pub(crate) fn of_table(name: &str) -> Option<Self> {
        [Self::Dependencies, Self::BuildOutput, Self::PythonEnvironments].into_iter().find(|c| c.name() == name)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::categories::Category;

        #[test]
        fn the_rust_build_cache_is_no_rows_category() {
            assert_eq!(Category::of_table("rust-build-cache"), None);
        }

        #[test]
        fn an_unknown_name_is_no_category() {
            assert_eq!(Category::of_table("dist"), None);
        }
    }

    mod positive {
        use crate::categories::Category;

        #[test]
        fn a_row_names_a_category_by_its_name() {
            assert_eq!(Category::of_table("python-environments"), Some(Category::PythonEnvironments));
        }

        #[test]
        fn an_id_is_the_module_and_the_name() {
            for category in Category::ALL {
                assert_eq!(category.id(), format!("stale-projects.{}", category.name()));
            }
        }
    }
}
