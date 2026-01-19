use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub ui: UiConfig,
    pub feeds: FeedsConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub startup_cleanup: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub show_ascii_banner: bool,
    #[serde(default = "default_tab")]
    pub default_tab: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeedsConfig {
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub sources: Vec<FeedSource>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeedSource {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub urls: Option<Vec<String>>,
    #[serde(default = "default_category")]
    pub category: String,
}

impl FeedSource {
    pub fn get_urls(&self) -> Vec<String> {
        let mut result = Vec::new();
        if let Some(ref url) = self.url {
            result.push(url.clone());
        }
        if let Some(ref urls) = self.urls {
            result.extend(urls.clone());
        }
        result
    }
}

fn default_theme() -> String {
    "catppuccin-mocha".to_string()
}

fn default_true() -> bool {
    true
}

fn default_tab() -> String {
    "all-posts".to_string()
}

fn default_category() -> String {
    "General".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            theme: default_theme(),
            startup_cleanup: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        UiConfig {
            show_ascii_banner: true,
            default_tab: default_tab(),
        }
    }
}

impl Default for FeedsConfig {
    fn default() -> Self {
        FeedsConfig {
            urls: vec![],
            sources: vec![],
        }
    }
}

pub fn load_config_from_path<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let path = path.as_ref();

    // Try to read existing config
    match fs::read_to_string(path) {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str)
                .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
            Ok(config)
        }
        Err(_) => {
            // Create default config if it doesn't exist
            let default_config = Config {
                app: AppConfig::default(),
                ui: UiConfig::default(),
                feeds: FeedsConfig {
                    urls: vec![],
                    sources: vec![
                        FeedSource {
                            url: Some("https://nesslabs.com/feed".to_string()),
                            urls: None,
                            category: "Productivity".to_string(),
                        },
                        FeedSource {
                            url: Some("https://dev.to/rss".to_string()),
                            urls: None,
                            category: "Technology".to_string(),
                        },
                        FeedSource {
                            url: Some("https://jamesclear.com/feed".to_string()),
                            urls: None,
                            category: "Productivity".to_string(),
                        },
                    ],
                },
            };

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).ok();
            }

            // Save the default config
            save_config_to_path(&default_config, path)?;
            eprintln!("Created default config at: {}", path.display());
            Ok(default_config)
        }
    }
}

#[allow(dead_code)]
pub fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    save_config_to_path(config, "config.toml")
}

#[allow(dead_code)]
pub fn save_config_to_path<P: AsRef<Path>>(config: &Config, path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }

    fs::write(path, config_str)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(())
}
