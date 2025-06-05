use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use clipboard::{ClipboardContext, ClipboardProvider};

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Editor configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// General editor settings
    #[serde(default)]
    pub editor: EditorConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// Keybindings
    #[serde(default)]
    pub keybindings: HashMap<String, String>,

    /// Plugin settings
    #[serde(default)]
    pub plugins: HashMap<String, serde_json::Value>,
}

/// Editor settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EditorConfig {
    /// Tab size
    #[serde(default = "default_tab_size")]
    pub tab_size: usize,

    /// Use spaces instead of tabs
    #[serde(default = "default_use_spaces")]
    pub use_spaces: bool,

    /// Show line numbers
    #[serde(default = "default_show_line_numbers")]
    pub show_line_numbers: bool,

    /// Highlight current line
    #[serde(default = "default_highlight_current_line")]
    pub highlight_current_line: bool,

    /// Word wrap
    #[serde(default = "default_word_wrap")]
    pub word_wrap: bool,

    /// Auto save
    #[serde(default = "default_auto_save")]
    pub auto_save: bool,

    /// Auto save delay in milliseconds
    #[serde(default = "default_auto_save_delay")]
    pub auto_save_delay: u64,
}

/// UI settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    /// Theme
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Font
    #[serde(default = "default_font")]
    pub font: String,

    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: usize,

    /// Show status bar
    #[serde(default = "default_show_status_bar")]
    pub show_status_bar: bool,

    /// Show menu bar
    #[serde(default = "default_show_menu_bar")]
    pub show_menu_bar: bool,

    /// Show minimap
    #[serde(default = "default_show_minimap")]
    pub show_minimap: bool,
}

// Default values
fn default_tab_size() -> usize {
    4
}
fn default_use_spaces() -> bool {
    true
}
fn default_show_line_numbers() -> bool {
    true
}
fn default_highlight_current_line() -> bool {
    true
}
fn default_word_wrap() -> bool {
    false
}
fn default_auto_save() -> bool {
    false
}
fn default_auto_save_delay() -> u64 {
    1000
}
fn default_theme() -> String {
    "default".to_string()
}
fn default_font() -> String {
    "Jetbrains Mono".to_string()
}
fn default_font_size() -> usize {
    12
}
fn default_show_status_bar() -> bool {
    true
}
fn default_show_menu_bar() -> bool {
    true
}
fn default_show_minimap() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            ui: UiConfig::default(),
            keybindings: HashMap::new(),
            plugins: HashMap::new(),
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_size: default_tab_size(),
            use_spaces: default_use_spaces(),
            show_line_numbers: default_show_line_numbers(),
            highlight_current_line: default_highlight_current_line(),
            word_wrap: default_word_wrap(),
            auto_save: default_auto_save(),
            auto_save_delay: default_auto_save_delay(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font: default_font(),
            font_size: default_font_size(),
            show_status_bar: default_show_status_bar(),
            show_menu_bar: default_show_menu_bar(),
            show_minimap: default_show_minimap(),
        }
    }
}

/// Configuration manager
pub struct ConfigManager {
    /// The config
    config: Config,

    /// The path to the config file
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new config manager
    pub fn new(config_dir: &Path) -> Self {
        let config_path = config_dir.join("config.json");

        Self {
            config: Config::default(),
            config_path,
        }
    }

    /// Load the config
    pub fn load(&mut self) -> Result<()> {
        // Create config directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Load config if it exists, otherwise use defaults
        if self.config_path.exists() {
            let config_str = fs::read_to_string(&self.config_path)?;
            self.config = serde_json::from_str(&config_str)
                .map_err(|e| anyhow!("Failed to parse config: {}", e))?;
        }

        Ok(())
    }

    /// Save the config
    pub fn save(&self) -> Result<()> {
        let config_str = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    /// Get the config
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Get a mutable reference to the config
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Update a setting
    pub fn update_setting(&mut self, path: &str, value: serde_json::Value) -> Result<()> {
        // Handle simple cases for common settings
        match path {
            "editor.tabSize" => {
                self.config.editor.tab_size =
                    value.as_u64().ok_or_else(|| anyhow!("Expected number"))? as usize;
            }
            "editor.useSpaces" => {
                self.config.editor.use_spaces =
                    value.as_bool().ok_or_else(|| anyhow!("Expected boolean"))?;
            }
            "editor.showLineNumbers" => {
                self.config.editor.show_line_numbers =
                    value.as_bool().ok_or_else(|| anyhow!("Expected boolean"))?;
            }
            "editor.highlightCurrentLine" => {
                self.config.editor.highlight_current_line =
                    value.as_bool().ok_or_else(|| anyhow!("Expected boolean"))?;
            }
            "editor.wordWrap" => {
                self.config.editor.word_wrap =
                    value.as_bool().ok_or_else(|| anyhow!("Expected boolean"))?;
            }
            "ui.theme" => {
                self.config.ui.theme = value
                    .as_str()
                    .ok_or_else(|| anyhow!("Expected string"))?
                    .to_string();
            }
            "ui.fontSize" => {
                self.config.ui.font_size =
                    value.as_u64().ok_or_else(|| anyhow!("Expected number"))? as usize;
            }
            _ => {
                // For plugin settings or more complex paths, we would need
                // a more sophisticated approach
                return Err(anyhow!("Unsupported setting path: {}", path));
            }
        }

        Ok(())
    }

    /// Get a setting by path
    pub fn get_setting(&self, path: &str) -> Result<serde_json::Value> {
        match path {
            "editor.tabSize" => Ok(serde_json::json!(self.config.editor.tab_size)),
            "editor.useSpaces" => Ok(serde_json::json!(self.config.editor.use_spaces)),
            "editor.showLineNumbers" => Ok(serde_json::json!(self.config.editor.show_line_numbers)),
            "editor.highlightCurrentLine" => {
                Ok(serde_json::json!(self.config.editor.highlight_current_line))
            }
            "editor.wordWrap" => Ok(serde_json::json!(self.config.editor.word_wrap)),
            "ui.theme" => Ok(serde_json::json!(self.config.ui.theme)),
            "ui.fontSize" => Ok(serde_json::json!(self.config.ui.font_size)),
            _ => Err(anyhow!("Unsupported setting path: {}", path)),
        }
    }
}
