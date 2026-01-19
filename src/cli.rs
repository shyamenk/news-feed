use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "news")]
#[command(author = "News Feed Contributors")]
#[command(version = "0.1.0")]
#[command(about = "A terminal-based RSS feed reader with a beautiful TUI", long_about = None)]
#[command(after_help = "EXAMPLES:
    news                          Start the RSS reader with default settings
    news --config ~/.config/news/config.toml
                                  Use a custom config file
    news --db-path ~/.local/share/news/feeds.db
                                  Use a custom database location
    news reset-db                 Reset the database (removes all feeds and posts)
    news export-feeds > feeds.opml
                                  Export feeds to OPML format
    news import-feeds feeds.opml  Import feeds from OPML file

KEYBINDINGS:
    Tab/Shift+Tab    Navigate between tabs
    1-6              Jump to specific tab
    j/Down           Move down in list
    k/Up             Move up in list
    Enter            Open article
    Esc/Backspace    Return to list view
    b                Toggle bookmark
    a                Toggle archive
    l                Toggle read later
    d                Delete item
    n/+              Add new feed/category
    q                Quit application

For more information, visit: https://github.com/yourusername/news-feed")]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Path to database file
    #[arg(short, long, value_name = "FILE")]
    pub db_path: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable automatic feed updates on startup
    #[arg(long)]
    pub no_auto_update: bool,

    /// Theme to use (overrides config file)
    #[arg(short, long, value_name = "THEME")]
    pub theme: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Reset the database (removes all feeds and posts)
    ResetDb {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Export feeds to OPML format
    ExportFeeds {
        /// Output file (defaults to stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Import feeds from OPML file
    ImportFeeds {
        /// Input OPML file
        #[arg(value_name = "FILE")]
        input: PathBuf,
    },

    /// Clean up old posts (older than specified days)
    Cleanup {
        /// Number of days to keep posts
        #[arg(short, long, default_value = "30")]
        days: u32,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Show configuration paths and information
    Info,

    /// List all feeds in the database
    ListFeeds,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Get the config path, using XDG Base Directory if not specified
    pub fn get_config_path(&self) -> PathBuf {
        if let Some(ref path) = self.config {
            path.clone()
        } else {
            Self::default_config_path()
        }
    }

    /// Get the database path, using XDG Base Directory if not specified
    pub fn get_db_path(&self) -> PathBuf {
        if let Some(ref path) = self.db_path {
            path.clone()
        } else {
            Self::default_db_path()
        }
    }

    /// Get default config path using XDG Base Directory specification
    pub fn default_config_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "news-feed", "news") {
            let config_dir = proj_dirs.config_dir();
            std::fs::create_dir_all(config_dir).ok();
            config_dir.join("config.toml")
        } else {
            // Fallback to current directory
            PathBuf::from("config.toml")
        }
    }

    /// Get default database path using XDG Base Directory specification
    pub fn default_db_path() -> PathBuf {
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "news-feed", "news") {
            let data_dir = proj_dirs.data_dir();
            std::fs::create_dir_all(data_dir).ok();
            data_dir.join("news_feed.db")
        } else {
            // Fallback to current directory
            PathBuf::from("news_feed.db")
        }
    }
}
