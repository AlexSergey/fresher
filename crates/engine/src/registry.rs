//! The modules the app runs: registered once at start, in a fixed order.

use std::sync::Arc;

use fresher_core::{
    AccessStatus, CategoryDescriptor, DescriptorError, Dictionary, DictionaryError, Module, ModuleDescriptor,
    ModuleGroup, ModuleId, ModuleInfo, SchemaError, SettingsSchema, ToolCommand, core_dictionary,
};

/// Every module of the app, with the settings schema and the dictionary they make together.
pub struct Registry {
    modules: Vec<Arc<dyn Module>>,
    schema: Option<SettingsSchema>,
    dictionary: Dictionary,
}

/// Why a module, or the core's own texts, were refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistryError {
    #[error("the core's dictionary is refused: {source}")]
    CoreDictionary { source: DictionaryError },
    #[error("module `{id}` is registered twice")]
    DuplicateModule { id: String },
    #[error("the dictionary of module `{id}` is refused: {source}")]
    Dictionary { id: String, source: DictionaryError },
    #[error("the settings of module `{id}` are refused: {source}")]
    Settings { id: String, source: SchemaError },
    #[error("the descriptor of module `{id}` is refused: {source}")]
    Descriptor { id: String, source: DescriptorError },
    #[error("module `{id}` is of the group Debug, which only a development build runs")]
    DebugModule { id: String },
}

impl Registry {
    /// A registry without modules; its dictionary starts with the core's texts, in front of the modules'.
    pub fn new() -> Result<Self, RegistryError> {
        let dictionary = core_dictionary().map_err(|source| RegistryError::CoreDictionary { source })?;
        Ok(Self { modules: Vec::new(), schema: None, dictionary })
    }

    /// Adds `module` after the ones already registered. A refused module leaves the registry as it was.
    pub fn register(&mut self, module: Box<dyn Module>) -> Result<(), RegistryError> {
        let descriptor = module.descriptor();
        let id = descriptor.id();
        if self.modules.iter().any(|known| known.descriptor().id() == id) {
            return Err(RegistryError::DuplicateModule { id: id.as_str().into() });
        }
        check_group(descriptor, cfg!(debug_assertions))?;
        descriptor.check_tools().map_err(|source| RegistryError::Descriptor { id: id.as_str().into(), source })?;
        let refused = |source| RegistryError::Dictionary { id: id.as_str().into(), source };
        let dictionary = Dictionary::parse(id.as_str(), descriptor.locales().en).map_err(refused)?;
        let schema = SettingsSchema::new(self.descriptors().chain([descriptor]))
            .map_err(|source| RegistryError::Settings { id: id.as_str().into(), source })?;
        self.dictionary.merge(dictionary).map_err(refused)?;
        self.schema = Some(schema);
        self.modules.push(Arc::from(module));
        tracing::debug!(module = id.as_str(), "module registered");
        Ok(())
    }

    /// The modules for the UI, in registration order, each category's availability worked out from `access`.
    pub fn modules(&self, access: &AccessStatus) -> Vec<ModuleInfo> {
        self.descriptors().map(|descriptor| ModuleInfo::new(descriptor, access)).collect()
    }

    /// The checks of the tools every module needs, in registration order.
    pub fn tool_checks(&self) -> impl Iterator<Item = &ToolCommand> {
        self.descriptors().flat_map(ModuleDescriptor::tool_checks)
    }

    /// Every category of every module, in registration order.
    pub fn categories(&self) -> impl Iterator<Item = &CategoryDescriptor> {
        self.descriptors().flat_map(ModuleDescriptor::categories)
    }

    /// The settings of the core and of every module. Registration already checked them, so this fails only if the
    /// core's own settings repeat a key.
    pub fn schema(&self) -> Result<SettingsSchema, SchemaError> {
        match &self.schema {
            Some(schema) => Ok(schema.clone()),
            None => SettingsSchema::new([]),
        }
    }

    /// The merged texts of the core and every module.
    pub const fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    pub(crate) fn registered(&self) -> &[Arc<dyn Module>] {
        &self.modules
    }

    /// The id of the registered module named `id`.
    pub(crate) fn module_id(&self, id: &str) -> Option<ModuleId> {
        self.descriptors().map(ModuleDescriptor::id).find(|known| known.as_str() == id)
    }

    /// Every registered module with its group, in registration order.
    pub(crate) fn module_groups(&self) -> impl Iterator<Item = (ModuleId, ModuleGroup)> {
        self.descriptors().map(|descriptor| (descriptor.id(), descriptor.group()))
    }

    /// The registered modules of `group`, in registration order.
    pub(crate) fn modules_of(&self, group: ModuleGroup) -> impl Iterator<Item = ModuleId> {
        self.descriptors().filter(move |descriptor| descriptor.group() == group).map(ModuleDescriptor::id)
    }

    fn descriptors(&self) -> impl Iterator<Item = &ModuleDescriptor> {
        self.modules.iter().map(|module| module.descriptor())
    }
}

/// Refuses a module of the group `Debug` unless `development_build`: that group never ships.
fn check_group(descriptor: &ModuleDescriptor, development_build: bool) -> Result<(), RegistryError> {
    match descriptor.group() {
        ModuleGroup::Debug if !development_build => {
            Err(RegistryError::DebugModule { id: descriptor.id().as_str().into() })
        }
        ModuleGroup::Development | ModuleGroup::Debug => Ok(()),
    }
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<_> = self.descriptors().map(ModuleDescriptor::id).collect();
        formatter.debug_struct("Registry").field("modules", &ids).finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{AccessStatus, DangerLevel, DictionaryError};

        use crate::testing::TestModule;
        use crate::{Registry, RegistryError};

        #[test]
        fn a_release_build_refuses_a_debug_module() {
            let module = TestModule::builder("alpha").group(fresher_core::ModuleGroup::Debug).build();

            let error = super::super::check_group(fresher_core::Module::descriptor(&module), false);

            assert_eq!(error, Err(RegistryError::DebugModule { id: "alpha".into() }));
        }

        #[test]
        fn refuses_a_second_module_with_the_same_id() {
            let mut registry = Registry::new().unwrap();
            registry.register(TestModule::builder("alpha").boxed()).unwrap();

            let error = registry.register(TestModule::builder("alpha").boxed());

            assert_eq!(error, Err(RegistryError::DuplicateModule { id: "alpha".into() }));
            assert_eq!(registry.modules(&AccessStatus::default()).len(), 1);
        }

        #[test]
        fn refuses_dictionary_keys_outside_the_module_namespace() {
            let mut registry = Registry::new().unwrap();
            let module = TestModule::builder("alpha").locales(r#"{ "beta.caches.name": "Caches" }"#);

            let error = registry.register(module.boxed());

            let source = DictionaryError::OutsideNamespace { key: "beta.caches.name".into() };
            assert_eq!(error, Err(RegistryError::Dictionary { id: "alpha".into(), source }));
            assert_eq!(registry.modules(&AccessStatus::default()), vec![]);
        }

        #[test]
        fn refuses_a_module_whose_dictionary_is_not_flat_json() {
            let mut registry = Registry::new().unwrap();
            let nested = TestModule::builder("alpha").locales(r#"{ "alpha": { "caches": "Caches" } }"#);
            let broken = TestModule::builder("beta").locales(r#"{ "beta.caches.name": "#);

            let nested = registry.register(nested.boxed());
            let broken = registry.register(broken.boxed());

            let source = DictionaryError::NotFlat { key: Some("alpha".into()) };
            assert_eq!(nested, Err(RegistryError::Dictionary { id: "alpha".into(), source }));
            assert!(matches!(broken, Err(RegistryError::Dictionary { source: DictionaryError::Json { .. }, .. })));
        }

        #[test]
        fn refuses_a_module_that_needs_a_tool_without_its_check() {
            let mut registry = Registry::new().unwrap();
            let docker = fresher_core::Capability::Tool(fresher_core::ToolId::new("docker").unwrap());
            let module = TestModule::builder("alpha").needing(
                "alpha.images",
                DangerLevel::Low,
                vec![fresher_core::Need::required(docker)],
            );

            let error = registry.register(module.boxed());

            let source = fresher_core::DescriptorError::MissingToolCheck { tool: "docker".into() };
            assert_eq!(error, Err(RegistryError::Descriptor { id: "alpha".into(), source }));
            assert_eq!(registry.tool_checks().count(), 0);
        }

        #[test]
        fn a_refused_module_leaves_the_registry_as_it_was() {
            let mut registry = Registry::new().unwrap();
            let good = TestModule::builder("alpha")
                .category("alpha.caches", DangerLevel::Safe)
                .locales(r#"{ "alpha.caches.name": "Caches" }"#);
            registry.register(good.boxed()).unwrap();
            let before =
                (registry.modules(&AccessStatus::default()), registry.dictionary().clone(), registry.schema().unwrap());

            let error = registry.register(TestModule::builder("beta").locales(r#"{ "beta.A": "A" }"#).boxed());

            assert!(error.is_err());
            assert_eq!(
                (registry.modules(&AccessStatus::default()), registry.dictionary().clone(), registry.schema().unwrap()),
                before
            );
        }
    }

    mod positive {
        use fresher_core::{AccessStatus, CategoryId, DangerLevel, ModuleGroup, ModuleId, core_dictionary};

        use crate::Registry;
        use crate::testing::TestModule;

        #[test]
        fn merges_the_dictionaries_of_two_modules() {
            let mut registry = Registry::new().unwrap();
            let alpha = TestModule::builder("alpha").locales(r#"{ "alpha.caches.name": "Alpha caches" }"#);
            let beta = TestModule::builder("beta").locales(r#"{ "beta.images.name": "Beta images" }"#);

            registry.register(alpha.boxed()).unwrap();
            registry.register(beta.boxed()).unwrap();

            let dictionary = registry.dictionary();
            assert_eq!(dictionary.len(), core_dictionary().unwrap().len() + 2);
            assert!(dictionary.get("core.error.busy").is_some());
            assert_eq!(dictionary.get("alpha.caches.name"), Some("Alpha caches"));
            assert_eq!(dictionary.get("beta.images.name"), Some("Beta images"));
        }

        #[test]
        fn lists_modules_in_registration_order() {
            let mut registry = Registry::new().unwrap();
            for id in ["gamma", "alpha", "beta"] {
                registry.register(TestModule::builder(id).boxed()).unwrap();
            }

            let modules: Vec<_> = registry
                .modules(&AccessStatus::default())
                .iter()
                .map(|module| (module.id.as_str(), module.group))
                .collect();

            let development = ModuleGroup::Development;
            assert_eq!(modules, [("gamma", development), ("alpha", development), ("beta", development)]);
        }

        #[test]
        fn a_development_build_takes_a_module_of_any_group() {
            for group in ModuleGroup::ALL {
                let module = TestModule::builder("alpha").group(group).build();

                assert_eq!(super::super::check_group(fresher_core::Module::descriptor(&module), true), Ok(()));
            }
            let release = TestModule::builder("alpha").group(ModuleGroup::Development).build();
            assert_eq!(super::super::check_group(fresher_core::Module::descriptor(&release), false), Ok(()));
        }

        #[test]
        fn collects_categories_and_settings_of_every_module() {
            let mut registry = Registry::new().unwrap();
            let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe);
            let beta = TestModule::builder("beta").category("beta.images", DangerLevel::High);

            registry.register(alpha.boxed()).unwrap();
            registry.register(beta.boxed()).unwrap();

            let categories: Vec<_> = registry.categories().map(|category| category.id().as_str()).collect();
            assert_eq!(categories, ["alpha.caches", "beta.images"]);
            let schema = registry.schema().unwrap();
            let keys: Vec<_> = schema.descriptors().iter().map(|setting| setting.key().as_str()).collect();
            assert_eq!(keys[keys.len() - 2..], ["alpha.enabled", "beta.enabled"]);
            assert_eq!(keys[0], "core.motion");
            let beta_images = CategoryId::new(ModuleId::new("beta").unwrap(), "beta.images").unwrap();
            assert_eq!(registry.modules(&AccessStatus::default())[1].categories[0].descriptor.id(), beta_images);
        }

        #[test]
        fn an_empty_registry_has_only_the_core_settings_and_texts() {
            let registry = Registry::new().unwrap();

            let schema = registry.schema().unwrap();

            assert!(schema.descriptors().iter().all(|setting| setting.key().namespace() == "core"));
            assert_eq!(schema.descriptors().len(), 10);
            assert_eq!(registry.dictionary().len(), core_dictionary().unwrap().len());
            assert_eq!(format!("{registry:?}"), "Registry { modules: [], .. }");
        }
    }
}
