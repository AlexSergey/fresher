//! The IDEs the `jetbrains` module knows, as data: `products.toml`, parsed and checked when the module is built.

use std::collections::BTreeSet;

use fresher_core::{AppIdError, BundleId};
use serde::Deserialize;

/// The table shipped with the module.
const SHIPPED: &str = include_str!("../products.toml");

/// What the vendor's bundle ids start with, compared ignoring case as bundle ids are.
const VENDOR_PREFIX: &str = "com.jetbrains.";

/// Why a products table was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProductsError {
    #[error("the products table is not valid TOML: {0}")]
    Format(String),
    #[error("the products table lists no product")]
    Empty,
    #[error("`{folder}` is not a folder name of plain letters")]
    NotLetters { folder: String },
    #[error("the folder name `{folder}` is listed twice")]
    DuplicateFolder { folder: String },
    #[error("the product `{folder}` has no name to show")]
    EmptyName { folder: String },
    #[error("the product `{folder}` has no bundle id")]
    NoBundleId { folder: String },
    #[error(transparent)]
    BundleId(#[from] AppIdError),
    #[error("the bundle id `{id}` is listed twice")]
    DuplicateBundleId { id: String },
}

/// One IDE: the name its version folders start with, the name shown and the bundle ids of its apps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Product {
    folder: String,
    name: String,
    bundle_ids: Vec<BundleId>,
}

impl Product {
    /// What each version's folder name starts with (`WebStorm`).
    pub(crate) fn folder(&self) -> &str {
        &self.folder
    }

    /// The product as the app shows it (`WebStorm`).
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    /// The bundle ids of its apps: the release's and, when there is one, the early access build's.
    pub(crate) fn bundle_ids(&self) -> &[BundleId] {
        &self.bundle_ids
    }
}

/// Every product, each folder name and bundle id in one product only, and the vendor's apps that are not IDEs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Products {
    products: Vec<Product>,
    other_apps: Vec<BundleId>,
}

impl Products {
    /// The table shipped with the module.
    pub(crate) fn shipped() -> Result<Self, ProductsError> {
        Self::parse(SHIPPED)
    }

    /// A table read from `text` and checked: at least one product; folder names of plain ASCII letters, unique; a name
    /// to show; at least one bundle id per product; every bundle id, the other apps' too, valid and listed once in the
    /// whole table.
    pub(crate) fn parse(text: &str) -> Result<Self, ProductsError> {
        let table: RawTable = toml::from_str(text).map_err(|error| ProductsError::Format(error.to_string()))?;
        if table.product.is_empty() {
            return Err(ProductsError::Empty);
        }
        let mut folders = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut products = Vec::with_capacity(table.product.len());
        for raw in table.product {
            if raw.folder.is_empty() || !raw.folder.chars().all(|char| char.is_ascii_alphabetic()) {
                return Err(ProductsError::NotLetters { folder: raw.folder });
            }
            if !folders.insert(raw.folder.clone()) {
                return Err(ProductsError::DuplicateFolder { folder: raw.folder });
            }
            if raw.name.trim().is_empty() {
                return Err(ProductsError::EmptyName { folder: raw.folder });
            }
            if raw.bundle_ids.is_empty() {
                return Err(ProductsError::NoBundleId { folder: raw.folder });
            }
            let mut bundle_ids = Vec::with_capacity(raw.bundle_ids.len());
            for id in raw.bundle_ids {
                let id = BundleId::new(id)?;
                if !ids.insert(id.clone()) {
                    return Err(ProductsError::DuplicateBundleId { id: id.as_str().into() });
                }
                bundle_ids.push(id);
            }
            products.push(Product { folder: raw.folder, name: raw.name, bundle_ids });
        }
        let mut other_apps = Vec::with_capacity(table.other_apps.len());
        for id in table.other_apps {
            let id = BundleId::new(id)?;
            if !ids.insert(id.clone()) {
                return Err(ProductsError::DuplicateBundleId { id: id.as_str().into() });
            }
            other_apps.push(id);
        }
        Ok(Self { products, other_apps })
    }

    /// The product whose folder name is exactly `folder`, case included.
    pub(crate) fn by_folder(&self, folder: &str) -> Option<&Product> {
        self.products.iter().find(|product| product.folder == folder)
    }

    /// Every product, in the table's order.
    #[cfg(test)]
    pub(crate) fn all(&self) -> &[Product] {
        &self.products
    }

    /// Whether `id` is the vendor's (`com.jetbrains.…`) but listed nowhere in the table: an IDE the table does not know
    /// yet, whose running the decision treats as an open IDE of every product.
    pub(crate) fn is_unknown_vendor_app(&self, id: &BundleId) -> bool {
        id.folded().starts_with(VENDOR_PREFIX)
            && !self.other_apps.contains(id)
            && !self.products.iter().any(|product| product.bundle_ids.contains(id))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawTable {
    #[serde(default)]
    other_apps: Vec<String>,
    #[serde(default)]
    product: Vec<RawProduct>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct RawProduct {
    folder: String,
    name: String,
    bundle_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{AppIdError, BundleId};

        use crate::products::{Products, ProductsError};

        fn product(folder: &str, ids: &str) -> String {
            format!("[[product]]\nfolder = \"{folder}\"\nname = \"{folder}\"\nbundle-ids = [{ids}]\n")
        }

        #[test]
        fn refuses_an_empty_list() {
            assert_eq!(Products::parse(""), Err(ProductsError::Empty));
        }

        #[test]
        fn refuses_text_that_is_not_a_table() {
            assert!(matches!(Products::parse("[[product]]\nfolder = 1"), Err(ProductsError::Format(_))));
            assert!(matches!(
                Products::parse(&format!("{}size = 3\n", product("WebStorm", "\"com.jetbrains.WebStorm\""))),
                Err(ProductsError::Format(_))
            ));
        }

        #[test]
        fn refuses_a_duplicate_folder() {
            let text =
                product("WebStorm", "\"com.jetbrains.WebStorm\"") + &product("WebStorm", "\"com.example.Other\"");

            assert_eq!(Products::parse(&text), Err(ProductsError::DuplicateFolder { folder: "WebStorm".into() }));
        }

        #[test]
        fn refuses_a_folder_that_is_not_plain_letters() {
            for folder in ["", "Web Storm", "WebStorm2", "Web-Storm", "WebStorm.", "Wébstorm"] {
                let text = product(folder, "\"com.jetbrains.WebStorm\"");

                assert_eq!(
                    Products::parse(&text),
                    Err(ProductsError::NotLetters { folder: folder.into() }),
                    "{folder}"
                );
            }
        }

        #[test]
        fn refuses_an_empty_name() {
            let text = "[[product]]\nfolder = \"WebStorm\"\nname = \" \"\nbundle-ids = [\"com.jetbrains.WebStorm\"]\n";

            assert_eq!(Products::parse(text), Err(ProductsError::EmptyName { folder: "WebStorm".into() }));
        }

        #[test]
        fn refuses_an_empty_id_list() {
            assert_eq!(
                Products::parse(&product("WebStorm", "")),
                Err(ProductsError::NoBundleId { folder: "WebStorm".into() })
            );
        }

        #[test]
        fn refuses_an_invalid_id() {
            assert_eq!(
                Products::parse(&product("WebStorm", "\"com jetbrains\"")),
                Err(ProductsError::BundleId(AppIdError::NotABundleId { id: "com jetbrains".into() }))
            );
        }

        #[test]
        fn refuses_an_id_in_two_products() {
            let text =
                product("WebStorm", "\"com.jetbrains.WebStorm\"") + &product("PhpStorm", "\"com.jetbrains.webstorm\"");

            assert_eq!(
                Products::parse(&text),
                Err(ProductsError::DuplicateBundleId { id: "com.jetbrains.webstorm".into() })
            );
        }

        #[test]
        fn refuses_an_id_listed_twice_in_one_product() {
            let text = product("WebStorm", "\"com.jetbrains.WebStorm\", \"com.jetbrains.WebStorm\"");

            assert_eq!(
                Products::parse(&text),
                Err(ProductsError::DuplicateBundleId { id: "com.jetbrains.WebStorm".into() })
            );
        }

        #[test]
        fn refuses_an_other_app_that_is_also_a_products_id_or_invalid() {
            let text = format!(
                "other-apps = [\"com.jetbrains.webstorm\"]\n{}",
                product("WebStorm", "\"com.jetbrains.WebStorm\"")
            );
            assert_eq!(
                Products::parse(&text),
                Err(ProductsError::DuplicateBundleId { id: "com.jetbrains.webstorm".into() })
            );

            let text =
                format!("other-apps = [\"com jetbrains\"]\n{}", product("WebStorm", "\"com.jetbrains.WebStorm\""));
            assert_eq!(
                Products::parse(&text),
                Err(ProductsError::BundleId(AppIdError::NotABundleId { id: "com jetbrains".into() }))
            );
        }

        #[test]
        fn a_known_or_another_vendors_id_is_not_an_unknown_vendor_app() {
            let products = Products::shipped().unwrap();

            for id in [
                "com.jetbrains.WebStorm",
                "COM.JETBRAINS.WEBSTORM-EAP",
                "com.jetbrains.toolbox",
                "com.example.jetbrains",
                "com.jetbrainsx.Foo",
                "org.jetbrains.jcef.helper",
                "com.jetbrains",
            ] {
                assert!(!products.is_unknown_vendor_app(&BundleId::new(id).unwrap()), "{id}");
            }
        }

        #[test]
        fn finds_no_product_by_another_case_or_a_part_of_its_folder() {
            let products = Products::shipped().unwrap();

            for folder in ["webstorm", "WEBSTORM", "Web", "WebStormX", ""] {
                assert_eq!(products.by_folder(folder), None, "{folder}");
            }
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use crate::products::{Product, Products};

        #[test]
        fn the_shipped_table_parses() {
            let products = Products::shipped().unwrap();

            let folders: Vec<_> = products.all().iter().map(Product::folder).collect();
            assert_eq!(
                folders,
                [
                    "IntelliJIdea",
                    "IdeaIC",
                    "WebStorm",
                    "PyCharm",
                    "PyCharmCE",
                    "PhpStorm",
                    "GoLand",
                    "CLion",
                    "RubyMine",
                    "Rider",
                    "DataGrip",
                    "DataSpell",
                    "RustRover",
                    "AppCode",
                    "Aqua",
                    "Writerside"
                ]
            );
        }

        #[test]
        fn a_product_has_its_name_and_its_release_and_early_access_ids() {
            let products = Products::shipped().unwrap();

            let webstorm = products.by_folder("WebStorm").unwrap();
            assert_eq!(webstorm.name(), "WebStorm");
            assert_eq!(
                webstorm.bundle_ids(),
                [
                    BundleId::new("com.jetbrains.WebStorm").unwrap(),
                    BundleId::new("com.jetbrains.WebStorm-EAP").unwrap()
                ]
            );
            assert_eq!(products.by_folder("IdeaIC").unwrap().name(), "IntelliJ IDEA Community Edition");
        }

        #[test]
        fn every_product_has_its_release_id_first_and_an_early_access_id() {
            let products = Products::shipped().unwrap();

            for product in products.all() {
                let early = format!("{}-eap", product.bundle_ids()[0].folded());
                assert!(product.bundle_ids().iter().any(|id| id.folded() == early), "{}", product.folder());
            }
        }

        #[test]
        fn a_vendor_app_the_table_does_not_list_is_unknown() {
            let products = Products::shipped().unwrap();

            for id in ["com.jetbrains.fleet", "com.jetbrains.gateway", "COM.JETBRAINS.NewIde"] {
                assert!(products.is_unknown_vendor_app(&BundleId::new(id).unwrap()), "{id}");
            }
        }
    }
}
