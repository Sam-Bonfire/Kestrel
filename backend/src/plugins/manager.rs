use std::path::PathBuf;
use tracing::{info, warn};
use std::sync::Arc;

use super::traits::{
    BrandingPayload, CalendarProvider, MailProvider, ProviderBranding, ProviderPlugin,
};
use super::wasm_runtime::WasmEngine;

/// Represents a loaded plugin and its capabilities.
pub struct LoadedPlugin {
    pub id: String,
    pub branding: BrandingPayload,
    /// Box<dyn ProviderPlugin> for dynamic dispatch to the actual plugin implementation.
    plugin: Box<dyn ProviderPlugin>,
}

impl LoadedPlugin {
    pub fn new(plugin: Box<dyn ProviderPlugin>) -> Self {
        let branding = plugin.get_branding();
        let id = plugin.id().to_string();
        Self {
            id,
            branding,
            plugin,
        }
    }

    /// Get a reference to the underlying plugin.
    pub fn as_mail_provider(&self) -> &dyn MailProvider {
        self.plugin.as_ref()
    }

    pub fn as_calendar_provider(&self) -> &dyn CalendarProvider {
        self.plugin.as_ref()
    }

    pub fn as_provider_branding(&self) -> &dyn ProviderBranding {
        self.plugin.as_ref()
    }
}

/// Manages loading and querying of provider plugins.
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    plugins_dir: Option<PathBuf>,
    wasm_engine: Arc<WasmEngine>,
}

impl PluginManager {
    /// Create a new, empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugins_dir: None,
            wasm_engine: Arc::new(WasmEngine::new().expect("Failed to initialize Wasmtime engine")),
        }
    }

    /// Set the plugins directory path.
    pub fn with_plugins_dir(mut self, dir: PathBuf) -> Self {
        self.plugins_dir = Some(dir);
        self
    }

    /// Register a plugin instance directly (for testing or built-in plugins).
    pub fn register(&mut self, plugin: Box<dyn ProviderPlugin>) {
        let loaded = LoadedPlugin::new(plugin);
        info!("Registered plugin: {} ({})", loaded.id, loaded.branding.name);
        self.plugins.push(loaded);
    }

    /// Discover and load all available plugins from the plugins directory.
    pub async fn load_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = match &self.plugins_dir {
            Some(d) => d.clone(),
            None => {
                let dir = std::env::var("PLUGINS_DIR")
                    .unwrap_or_else(|_| "./plugins".to_string());
                PathBuf::from(dir)
            }
        };

        info!("Loading plugins from {}", dir.display());

        if !dir.exists() {
            warn!("Plugins directory {} does not exist, skipping plugin load", dir.display());
            return Ok(());
        }

        let entries = std::fs::read_dir(&dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                info!("Discovered plugin WASM: {}", path.display());
                
                // Read the component
                match wasmtime::component::Component::from_file(&self.wasm_engine.engine, &path) {
                    Ok(component) => {
                        // Extract plugin ID from filename (e.g., "google.wasm" -> "google")
                        let id = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        
                        // Instantiate asynchronously
                        // Since load_all is async, we can do this!
                        match super::wasm_plugin::WasmPlugin::new(id.clone(), self.wasm_engine.clone(), component).await {
                            Ok(plugin) => {
                                info!("WASM Plugin '{}' loaded successfully.", id);
                                self.register(Box::new(plugin));
                            }
                            Err(e) => {
                                warn!("Failed to initialize WASM plugin {}: {}", path.display(), e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to compile WASM component {}: {}", path.display(), e);
                    }
                }
            }
        }

        info!(
            "Plugin loading complete: {} plugin(s) registered",
            self.plugins.len()
        );
        Ok(())
    }

    /// Return the number of loaded plugins.
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if any plugins are loaded.
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// Return all currently loaded branding plugins.
    pub fn get_brandings(&self) -> Vec<&BrandingPayload> {
        self.plugins.iter().map(|p| &p.branding).collect()
    }

    /// Find a plugin by its ID.
    pub fn find_by_id(&self, id: &str) -> Option<&LoadedPlugin> {
        self.plugins.iter().find(|p| p.id == id)
    }

    /// Find a plugin by provider name (case-insensitive match against branding.name).
    pub fn find_by_provider(&self, provider: &str) -> Option<&LoadedPlugin> {
        let lower = provider.to_lowercase();
        self.plugins
            .iter()
            .find(|p| p.branding.name.to_lowercase() == lower)
    }

    /// Return all loaded plugins.
    pub fn plugins(&self) -> &[LoadedPlugin] {
        &self.plugins
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::mock::MockProviderPlugin;

    #[test]
    fn test_register_and_list() {
        let mut manager = PluginManager::new();
        let mock = MockProviderPlugin::new("test-plugin", "Test Plugin");
        manager.register(Box::new(mock));

        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());

        let brandings = manager.get_brandings();
        assert_eq!(brandings.len(), 1);
        assert_eq!(brandings[0].name, "Test Plugin");
    }

    #[test]
    fn test_find_by_id() {
        let mut manager = PluginManager::new();
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        manager.register(Box::new(mock));

        assert!(manager.find_by_id("gmail").is_some());
        assert!(manager.find_by_id("outlook").is_none());
    }

    #[test]
    fn test_find_by_provider() {
        let mut manager = PluginManager::new();
        let mock = MockProviderPlugin::new("gmail", "Gmail");
        manager.register(Box::new(mock));

        assert!(manager.find_by_provider("Gmail").is_some());
        assert!(manager.find_by_provider("GMAIL").is_some());
        assert!(manager.find_by_provider("Outlook").is_none());
    }

    #[test]
    fn test_multiple_plugins() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(MockProviderPlugin::new("gmail", "Gmail")));
        manager.register(Box::new(MockProviderPlugin::new("outlook", "Outlook")));

        assert_eq!(manager.len(), 2);

        let brandings = manager.get_brandings();
        let names: Vec<&str> = brandings.iter().map(|b| b.name.as_str()).collect();
        assert!(names.contains(&"Gmail"));
        assert!(names.contains(&"Outlook"));
    }
}
