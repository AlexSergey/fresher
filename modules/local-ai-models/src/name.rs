//! A model's name read from its folder's path (design, section 3): by path components, never by slicing the path.

use std::path::{Component, Path};

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
/// - `PublisherModel`: the last two components, neither starting with `.` (title `publisher/model`);
/// - `OllamaStore`: a manifest's path, as [`OllamaTag::read`] reads it.
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
        Shape::OllamaStore => OllamaTag::read(path).map(|tag| ModelName { kind: None, title: tag.title() }),
    }
}

/// The host and the namespace of Ollama's own registry, whose models are shown by `model:tag` alone.
const OLLAMA_HOST: &str = "registry.ollama.ai";
const OLLAMA_NAMESPACE: &str = "library";

/// An Ollama model's tag: the last four components of a manifest's path under `manifests/`, ordered by them in turn
/// (the name order of the module's findings).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct OllamaTag {
    pub(crate) host: String,
    pub(crate) namespace: String,
    pub(crate) model: String,
    pub(crate) tag: String,
}

impl OllamaTag {
    /// The tag a manifest's `path` names: its last four components, each a name in UTF-8 not starting with `.`;
    /// `None` for fewer or any other kind of component.
    pub(crate) fn read(path: &Path) -> Option<Self> {
        let mut names = path.components().rev().take(4).map(|part| match part {
            Component::Normal(name) => name.to_str().filter(|name| !name.starts_with('.')),
            _ => None,
        });
        let (tag, model, namespace, host) = (names.next()??, names.next()??, names.next()??, names.next()??);
        Some(Self { host: host.into(), namespace: namespace.into(), model: model.into(), tag: tag.into() })
    }

    /// The title Ollama itself shows: `model:tag` for its registry's `library`, `namespace/model:tag` for its
    /// registry's other namespaces, `host/namespace/model:tag` for any other host.
    pub(crate) fn title(&self) -> String {
        let Self { host, namespace, model, tag } = self;
        match (host.as_str(), namespace.as_str()) {
            (OLLAMA_HOST, OLLAMA_NAMESPACE) => format!("{model}:{tag}"),
            (OLLAMA_HOST, _) => format!("{namespace}/{model}:{tag}"),
            _ => format!("{host}/{namespace}/{model}:{tag}"),
        }
    }

    /// What tells the model apart in the module's findings: `ollama/<host>/<namespace>/<model>/<tag>`.
    pub(crate) fn identity(&self) -> String {
        format!("ollama/{}/{}/{}/{}", self.host, self.namespace, self.model, self.tag)
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

        use crate::name::{OllamaTag, read};
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

        #[test]
        fn a_manifest_path_with_three_components_is_not_a_tag() {
            for path in ["/registry.ollama.ai/library/llama3", "library/llama3/latest", "/latest", "/"] {
                assert_eq!(OllamaTag::read(Path::new(path)), None, "{path}");
            }
        }

        #[test]
        fn a_hidden_component_is_not_a_tag() {
            for path in [
                "/m/manifests/.hidden/library/llama3/latest",
                "/m/manifests/registry.ollama.ai/.library/llama3/latest",
                "/m/manifests/registry.ollama.ai/library/.llama3/latest",
                "/m/manifests/registry.ollama.ai/library/llama3/.DS_Store",
            ] {
                assert_eq!(OllamaTag::read(Path::new(path)), None, "{path}");
            }
        }

        #[test]
        fn a_component_not_in_utf8_is_not_a_tag() {
            let bad = OsStr::from_bytes(b"lla\xffma");
            let folder = Path::new("/m/manifests/registry.ollama.ai/library");

            assert_eq!(OllamaTag::read(&folder.join(bad).join("latest")), None);
            assert_eq!(OllamaTag::read(&folder.join("llama3").join(bad)), None);
        }

        #[test]
        fn an_ollama_store_folder_is_not_a_model() {
            assert_eq!(read(Shape::OllamaStore, Path::new("/home/.ollama/models")), None);
        }
    }

    mod positive {
        use std::path::Path;

        use crate::name::{ModelName, OllamaTag, read};
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

        fn tag(path: &str) -> OllamaTag {
            OllamaTag::read(Path::new(path)).expect("a manifest path")
        }

        #[test]
        fn the_library_of_ollama_s_registry_is_titled_model_and_tag() {
            let found = tag("/home/.ollama/models/manifests/registry.ollama.ai/library/llama3/latest");

            assert_eq!(found.title(), "llama3:latest");
            assert_eq!(found.identity(), "ollama/registry.ollama.ai/library/llama3/latest");
            assert_eq!(
                read(Shape::OllamaStore, Path::new("/m/manifests/registry.ollama.ai/library/llama3/8b")),
                Some(named(None, "llama3:8b"))
            );
        }

        #[test]
        fn another_namespace_of_the_registry_is_titled_with_it() {
            let found = tag("/m/manifests/registry.ollama.ai/jmorganca/phi/q4");

            assert_eq!(found.title(), "jmorganca/phi:q4");
        }

        #[test]
        fn another_host_is_titled_with_it() {
            let found = tag("/m/manifests/hf.co/bartowski/gemma/Q4_K_M");

            assert_eq!(found.title(), "hf.co/bartowski/gemma:Q4_K_M");
            assert_eq!(found.identity(), "ollama/hf.co/bartowski/gemma/Q4_K_M");
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
