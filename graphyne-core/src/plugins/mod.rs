//! Plugin system for Graphyne
//!
//! This module provides a basic plugin system with dynamic library loading.
//! Plugins can extend Graphyne's functionality through lifecycle hooks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::ffi::OsStr;

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use crate::error::{GraphyneError, Result};

/// Trait that all Graphyne plugins must implement
pub trait GraphynePlugin: Send + Sync {
    /// Returns the name of the plugin
    fn name(&self) -> &str;
    
    /// Returns the version of the plugin
    fn version(&self) -> &str;
    
    /// Called when the plugin is loaded
    fn on_load(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when the plugin is unloaded
    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Hook called before search operations
    fn pre_search(&self, _query: &str) -> Result<()> {
        Ok(())
    }
    
    /// Hook called after search operations
    fn post_search(&self, _results: &mut Vec<(String, f32)>) -> Result<()> {
        Ok(())
    }
}

/// Plugin manager that handles loading, unloading, and lifecycle management
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn GraphynePlugin>>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    /// Create a new plugin manager with the specified plugin directory
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dir,
        }
    }
    
    /// Load a plugin by name
    ///
    /// Looks for a shared library file (.so, .dylib, or .dll) in the plugin directory.
    /// The file should be named according to the plugin name with the appropriate extension.
    pub fn load_plugin(&mut self, name: &str) -> Result<()> {
        let plugin_path = self.find_plugin_file(name)?;
        
        // In a real implementation, you would use libloading or similar to load the shared library
        // For now, we'll return an error indicating this is not fully implemented
        Err(GraphyneError::Other(
            format!("Dynamic plugin loading not yet implemented. Looking for: {:?}", plugin_path)
        ))
    }
    
    /// Unload a plugin by name
    ///
    /// Calls the plugin's on_unload hook and removes it from the plugin map.
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.remove(name) {
            plugin.on_unload()?;
            tracing::info!("Unloaded plugin: {}", name);
            Ok(())
        } else {
            Err(GraphyneError::Other(format!("Plugin not found: {}", name)))
        }
    }
    
    /// Register a plugin (for statically linked plugins)
    pub fn register_plugin(&mut self, plugin: Box<dyn GraphynePlugin>) -> Result<()> {
        let name = plugin.name().to_string();
        tracing::info!("Registering plugin: {} v{}", name, plugin.version());
        plugin.on_load()?;
        self.plugins.insert(name, plugin);
        Ok(())
    }
    
    /// Get a reference to a loaded plugin
    pub fn get_plugin(&self, name: &str) -> Option<&dyn GraphynePlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }
    
    /// Get a mutable reference to a loaded plugin
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut dyn GraphynePlugin> {
        self.plugins.get_mut(name).map(|p| p.as_mut())
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<(String, String)> {
        self.plugins
            .iter()
            .map(|(name, plugin)| (name.clone(), plugin.version().to_string()))
            .collect()
    }
    
    /// Call pre_search hook for all plugins
    pub fn pre_search_all(&self, query: &str) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.pre_search(query)?;
        }
        Ok(())
    }
    
    /// Call post_search hook for all plugins
    pub fn post_search_all(&self, results: &mut Vec<(String, f32)>) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.post_search(results)?;
        }
        Ok(())
    }
    
    /// Find the plugin file on disk
    fn find_plugin_file(&self, name: &str) -> Result<PathBuf> {
        let extensions = if cfg!(target_os = "windows") {
            vec!["dll"]
        } else if cfg!(target_os = "macos") {
            vec!["dylib", "so"]
        } else {
            vec!["so", "dylib"]
        };
        
        for ext in extensions {
            let filename = format!("{}.{}", name, ext);
            let path = self.plugin_dir.join(&filename);
            if path.exists() {
                return Ok(path);
            }
        }
        
        Err(GraphyneError::Other(format!(
            "Plugin file not found for '{}' in {:?}",
            name, self.plugin_dir
        )))
    }
    
    /// Scan the plugin directory and load all valid plugins
    pub fn load_all_plugins(&mut self) -> Result<()> {
        if !self.plugin_dir.exists() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.plugin_dir)
            .map_err(|e| GraphyneError::Other(format!("Failed to read plugin directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| GraphyneError::Other(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();
            
            if self.is_plugin_file(&path) {
                if let Some(name) = self.extract_plugin_name(&path) {
                    if let Err(e) = self.load_plugin(&name) {
                        tracing::warn!("Failed to load plugin '{}': {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if a path is a valid plugin file
    fn is_plugin_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        
        let ext = path.extension().and_then(OsStr::to_str);
        
        #[cfg(target_os = "windows")]
        return ext == Some("dll");
        
        #[cfg(target_os = "macos")]
        return ext == Some("dylib") || ext == Some("so");
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return ext == Some("so") || ext == Some("dylib");
    }
    
    /// Extract plugin name from file path
    fn extract_plugin_name(&self, path: &Path) -> Option<String> {
        let file_name = path.file_stem()?.to_str()?;
        Some(file_name.to_string())
    }
}

/// Example plugin implementation for testing
#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestPlugin {
        name: String,
        version: String,
    }
    
    impl TestPlugin {
        fn new(name: &str, version: &str) -> Self {
            Self {
                name: name.to_string(),
                version: version.to_string(),
            }
        }
    }
    
    impl GraphynePlugin for TestPlugin {
        fn name(&self) -> &str {
            &self.name
        }
        
        fn version(&self) -> &str {
            &self.version
        }
        
        fn on_load(&mut self) -> Result<()> {
            tracing::info!("TestPlugin {} loaded", self.name);
            Ok(())
        }
        
        fn on_unload(&mut self) -> Result<()> {
            tracing::info!("TestPlugin {} unloaded", self.name);
            Ok(())
        }
    }
    
    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new(PathBuf::from("./plugins"));
        
        let plugin = Box::new(TestPlugin::new("test-plugin", "1.0.0"));
        assert!(manager.register_plugin(plugin).is_ok());
        
        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].0, "test-plugin");
        assert_eq!(plugins[0].1, "1.0.0");
    }
    
    #[test]
    fn test_plugin_unload() {
        let mut manager = PluginManager::new(PathBuf::from("./plugins"));
        
        let plugin = Box::new(TestPlugin::new("test-plugin", "1.0.0"));
        manager.register_plugin(plugin).unwrap();
        
        assert!(manager.unload_plugin("test-plugin").is_ok());
        assert_eq!(manager.list_plugins().len(), 0);
    }
}
