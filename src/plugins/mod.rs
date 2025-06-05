//! Plugin system for the editor
//! This allows the editor to be extremely hackable, like VS Code
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A plugin for the editor
pub struct Plugin {
    /// The ID of the plugin
    pub id: String,

    /// The name of the plugin
    pub name: String,

    /// The version of the plugin
    pub version: String,

    /// The description of the plugin
    pub description: String,

    /// The path to the plugin directory
    pub path: PathBuf,

    /// The configuration for the plugin
    pub config: PluginConfig,

    /// The commands provided by the plugin
    pub commands: HashMap<String, Arc<dyn PluginCommand>>,
}

/// A command provided by a plugin
pub trait PluginCommand: Send + Sync {
    /// Execute the command
    fn execute(&self, args: &[String]) -> Result<()>;

    /// Get the name of the command
    fn name(&self) -> &str;

    /// Get the description of the command
    fn description(&self) -> &str;
}

/// Plugin configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    /// The ID of the plugin
    pub id: String,

    /// The name of the plugin
    pub name: String,

    /// The version of the plugin
    pub version: String,

    /// The description of the plugin
    pub description: String,

    /// The commands provided by the plugin
    pub commands: Vec<CommandConfig>,

    /// The keybindings provided by the plugin
    pub keybindings: Vec<KeybindingConfig>,

    /// Additional configuration options
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// Configuration for a command
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandConfig {
    /// The ID of the command
    pub id: String,

    /// The name of the command
    pub name: String,

    /// The description of the command
    pub description: String,
}

/// Configuration for a keybinding
#[derive(Debug, Serialize, Deserialize)]
pub struct KeybindingConfig {
    /// The key sequence
    pub key: String,

    /// The command to execute
    pub command: String,

    /// When the keybinding is active
    pub when: Option<String>,
}

/// Plugin manager
pub struct PluginManager {
    /// The plugins
    plugins: HashMap<String, Plugin>,

    /// The path to the plugins directory
    plugins_dir: PathBuf,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins: HashMap::new(),
            plugins_dir,
        }
    }

    /// Load all plugins
    pub fn load_plugins(&mut self) -> Result<()> {
        // Create plugins directory if it doesn't exist
        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)?;
        }

        // Iterate over plugin directories
        for entry in fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                match self.load_plugin(&path) {
                    Ok(plugin) => {
                        self.plugins.insert(plugin.id.clone(), plugin);
                    }
                    Err(e) => {
                        eprintln!("Failed to load plugin from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a plugin from a directory
    fn load_plugin(&self, path: &Path) -> Result<Plugin> {
        // Find the plugin.json file
        let config_path = path.join("plugin.json");
        if !config_path.exists() {
            return Err(anyhow!("plugin.json not found in {}", path.display()));
        }

        // Load the plugin config
        let config_str = fs::read_to_string(&config_path)?;
        let config: PluginConfig = serde_json::from_str(&config_str)?;

        // For now, just create a placeholder plugin
        // In a real implementation, we would load the plugin code and commands
        let plugin = Plugin {
            id: config.id.clone(),
            name: config.name.clone(),
            version: config.version.clone(),
            description: config.description.clone(),
            path: path.to_owned(),
            config,
            commands: HashMap::new(),
        };

        Ok(plugin)
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<&Plugin> {
        self.plugins.get(id)
    }

    /// Get all plugins
    pub fn get_plugins(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    /// Install a plugin from a path
    pub fn install_plugin(&mut self, source_path: &Path) -> Result<String> {
        // Ensure the plugin has a plugin.json
        let config_path = source_path.join("plugin.json");
        if !config_path.exists() {
            return Err(anyhow!(
                "plugin.json not found in {}",
                source_path.display()
            ));
        }

        // Load the plugin config to get the ID
        let config_str = fs::read_to_string(&config_path)?;
        let config: PluginConfig = serde_json::from_str(&config_str)?;

        // Create the destination directory
        let dest_dir = self.plugins_dir.join(&config.id);
        if dest_dir.exists() {
            return Err(anyhow!("Plugin with ID {} already installed", config.id));
        }

        // Copy the plugin files
        fs::create_dir_all(&dest_dir)?;
        Self::copy_dir_contents(source_path, &dest_dir)?;

        // Load the plugin
        let plugin = self.load_plugin(&dest_dir)?;
        self.plugins.insert(plugin.id.clone(), plugin);

        Ok(config.id)
    }

    /// Copy the contents of a directory
    fn copy_dir_contents(src: &Path, dst: &Path) -> Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(path.file_name().unwrap());

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                Self::copy_dir_contents(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Uninstall a plugin
    pub fn uninstall_plugin(&mut self, id: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(id) {
            fs::remove_dir_all(plugin.path)?;
            Ok(())
        } else {
            Err(anyhow!("Plugin with ID {} not found", id))
        }
    }
}
