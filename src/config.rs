use serde::{Deserialize, Serialize};
use std::fs;
use std::error::Error;

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
    pub url: String,
    #[serde(default = "default_category")]
    pub category: String,
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

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    // Try to read existing config
    match fs::read_to_string("config.toml") {
        Ok(config_str) => {
            let config: Config = toml::from_str(&config_str)
                .map_err(|e| format!("Failed to parse config.toml: {}", e))?;
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
                            url: "https://nesslabs.com/feed".to_string(),
                            category: "Productivity".to_string(),
                        },
                        FeedSource {
                            url: "https://dev.to/rss".to_string(),
                            category: "Technology".to_string(),
                        },
                        FeedSource {
                            url: "https://jamesclear.com/feed".to_string(),
                            category: "Productivity".to_string(),
                        },
                    ],
                },
            };

            // Save the default config
            save_config(&default_config)?;
            eprintln!("Created default config.toml");
            Ok(default_config)
        }
    }
}

#[allow(dead_code)]
pub fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let config_str = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write("config.toml", config_str)
        .map_err(|e| format!("Failed to write config.toml: {}", e))?;
    Ok(())
}
