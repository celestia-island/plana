use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::LlmProvider;

pub struct ProviderRegistry {
    providers: RwLock<HashMap<String, Arc<Box<dyn LlmProvider>>>>,
}

impl ProviderRegistry {
    fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    pub fn global() -> Arc<Self> {
        static INSTANCE: std::sync::OnceLock<Arc<ProviderRegistry>> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(Self::new())).clone()
    }

    pub fn register(&self, name: &str, provider: Box<dyn LlmProvider>) {
        let mut providers = self.providers.write().unwrap_or_else(|e| e.into_inner());
        providers.insert(name.to_string(), Arc::new(provider));
    }

    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn LlmProvider>>> {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers.get(name).cloned()
    }

    pub fn list_providers(&self) -> Vec<String> {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers.keys().cloned().collect()
    }

    pub fn exists(&self, name: &str) -> bool {
        let providers = self.providers.read().unwrap_or_else(|e| e.into_inner());
        providers.contains_key(name)
    }
}
