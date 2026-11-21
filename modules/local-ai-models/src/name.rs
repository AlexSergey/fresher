//! A model's name read from its folder's path (design, section 3): by path components, never by slicing the path.

use std::path::Path;

use crate::stores::Shape;

/// What separates the parts of a hub repository folder's name.
const SEPARATOR: &str = "--";

/// The name of one model: the kind its folder's name starts with (a hub repository only) and the title it is shown by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModelName {
    pub(crate) kind: Option<String>,
    pub(crate) title: String,
}

/// The name of the model whose folder is `path` in a store of `shape`; `None` when the folder's name does not fit the
/// shape or is not valid UTF-8:
///
/// - `HubRepo`: the last component is `<kind>--<owner>--<name>` (title `owner/name`) or `<kind>--<name>` (title
///   `name`), every part non-empty;
/// - `PublisherModel`: the last two components, neither starting with `.` (title `publisher/model`).
pub(crate) fn read(shape: Shape, path: &Path) -> Option<ModelName> {
    match shape {
        Shape::HubRepo => hub_repo(path.file_name()?.to_str()?),
        Shape::PublisherModel => {
            let model = path.file_name()?.to_str()?;
            let publisher = path.parent()?.file_name()?.to_str()?;
            if [publisher, model].iter().any(|part| part.starts_with('.')) {
                return None;
            }
            Some(ModelName { kind: None, title: format!("{publisher}/{model}") })
        }
    }
}

fn hub_repo(name: &str) -> Option<ModelName> {
    let parts: Vec<&str> = name.split(SEPARATOR).collect();
    if parts.iter().any(|part| part.is_empty()) {
        return None;
    }
    let (kind, title) = match parts.as_slice() {
        [kind, name] => (kind, (*name).to_owned()),
        [kind, owner, name] => (kind, format!("{owner}/{name}")),
        _ => return None,
    };
    Some(ModelName { kind: Some((*kind).to_owned()), title })
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt as _;
        use std::path::Path;

        use crate::name::read;
        use crate::stores::Shape;

        const HUB: &str = "/home/.cache/huggingface/hub";

        #[test]
        fn an_empty_part_is_not_a_repo() {
            for name in ["models--", "models----x", "--owner--name", "models--owner--", "--"] {
                assert_eq!(read(Shape::HubRepo, &Path::new(HUB).join(name)), None, "{name}");
            }
        }

        #[test]
        fn three_separators_are_not_a_repo() {
            assert_eq!(read(Shape::HubRepo, &Path::new(HUB).join("models--a--b--c")), None);
        }

        #[test]
        fn the_hub_s_own_entries_are_not_repos() {
            for name in [".locks", "version.txt"] {
                assert_eq!(read(Shape::HubRepo, &Path::new(HUB).join(name)), None, "{name}");
            }
            assert_eq!(read(Shape::HubRepo, Path::new("/")), None);
        }

        #[test]
        fn a_hidden_publisher_or_model_is_not_a_model() {
            for path in [
                "/home/.lmstudio/models/.cache/model",
                "/home/.lmstudio/models/publisher/.partial",
                "/home/.lmstudio/models/.a/.b",
            ] {
                assert_eq!(read(Shape::PublisherModel, Path::new(path)), None, "{path}");
            }
            assert_eq!(read(Shape::PublisherModel, Path::new("/model")), None);
        }

        #[test]
        fn a_name_not_in_utf8_is_not_a_model() {
            let bad = OsStr::from_bytes(b"models--owner--n\xffme");
            assert_eq!(read(Shape::HubRepo, &Path::new(HUB).join(bad)), None);

            let models = Path::new("/home/.lmstudio/models");
            let publisher = OsStr::from_bytes(b"pub\xfflisher");
            assert_eq!(read(Shape::PublisherModel, &models.join(publisher).join("model")), None);
            assert_eq!(read(Shape::PublisherModel, &models.join("publisher").join(publisher)), None);
        }
    }

    mod positive {
        use std::path::Path;

        use crate::name::{ModelName, read};
        use crate::stores::Shape;

        fn hub(name: &str) -> Option<ModelName> {
            read(Shape::HubRepo, &Path::new("/home/.cache/huggingface/hub").join(name))
        }

        fn named(kind: Option<&str>, title: &str) -> ModelName {
            ModelName { kind: kind.map(str::to_owned), title: title.to_owned() }
        }

        #[test]
        fn a_repo_with_an_owner_reads_owner_and_name() {
            assert_eq!(
                hub("models--google-bert--bert-base-uncased"),
                Some(named(Some("models"), "google-bert/bert-base-uncased"))
            );
        }

        #[test]
        fn a_repo_without_an_owner_reads_its_name() {
            assert_eq!(hub("models--gpt2"), Some(named(Some("models"), "gpt2")));
        }

        #[test]
        fn a_dataset_reads_its_kind() {
            assert_eq!(hub("datasets--squad"), Some(named(Some("datasets"), "squad")));
            assert_eq!(hub("spaces--owner--demo"), Some(named(Some("spaces"), "owner/demo")));
        }

        #[test]
        fn an_lm_studio_folder_reads_publisher_and_model() {
            let path = Path::new("/home/.lmstudio/models/lmstudio-community/Qwen2.5-7B-Instruct-GGUF");

            assert_eq!(
                read(Shape::PublisherModel, path),
                Some(named(None, "lmstudio-community/Qwen2.5-7B-Instruct-GGUF"))
            );
        }
    }
}
