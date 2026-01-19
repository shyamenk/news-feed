# News Feed TUI (News)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)](https://www.rust-lang.org/)

A fast, distraction-free, terminal-based RSS news feed reader built with Rust and Ratatui. Designed for developers who want to stay updated without leaving the command line.

## Features

- **Two-Pane Layout**: Sidebar navigation + posts list for intuitive browsing
- **Keyboard-First**: Vim-style navigation (`h/j/k/l`) with full keyboard control
- **Smart Views**: Fresh (unread), Starred, Read Later, Archived
- **Categories**: Organize feeds by category with lazy loading
- **Lazy Loading**: Only fetches data when a category is selected
- **Read State Tracking**: Read posts automatically hide from Fresh view
- **Clipboard Support**: Copy URLs with OSC52 (works in most terminals)
- **Offline-Friendly**: Feeds cached locally in SQLite database
- **Customizable Themes**: Catppuccin Mocha, Claude Code themes included

## Quick Start

```bash
git clone <repository_url>
cd news-feed
chmod +x install.sh
./install.sh
news
```

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `h` / `l` | Focus left/right pane |
| `j` / `k` | Navigate up/down |
| `Enter` | Select item / Open article |
| `Esc` | Go back / Cancel |
| `Tab` | Switch focus between panes |

### Actions
| Key | Action |
|-----|--------|
| `b` | Toggle bookmark/star |
| `l` | Toggle read later |
| `a` | Archive (in article) / Add (in sidebar) |
| `m` | Toggle read/unread |
| `d` | Delete (with confirmation) |
| `r` | Refresh feeds |
| `u` | Toggle show/hide read posts |

### Article View
| Key | Action |
|-----|--------|
| `j` / `k` | Scroll content |
| `PgUp` / `PgDn` | Scroll faster |
| `o` | Open in browser |
| `y` | Copy URL to clipboard |

### General
| Key | Action |
|-----|--------|
| `?` | Show help overlay |
| `q` | Quit application |

## UI Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ 󰑫 News Reader                                     [Fresh]       │
├────────────────────┬────────────────────────────────────────────┤
│  VIEWS             │  Posts                                     │
│   Fresh (12)      │                                            │
│  ★ Starred (3)     │  ▶ ● How to build TUIs in Rust     01/19   │
│   Later (5)       │    ○ Understanding async/await     01/18   │
│  󰆧 Archive (2)     │    ● New Ratatui features          01/18   │
│                    │                                            │
│  CATEGORIES        │                                            │
│  ▶ Tech (15)       │                                            │
│    Security (8)    │                                            │
│    General (5)     │                                            │
├────────────────────┴────────────────────────────────────────────┤
│ h/l:Focus │ j/k:Nav │ Enter:Open │ b:Star │ ?:Help │ q:Quit     │
└─────────────────────────────────────────────────────────────────┘
```

### Post Indicators
- `●` Unread post
- `○` Read post
- `★` Starred/bookmarked
- `󰃰` Saved for later
- `󰆧` Archived

## First Run Setup

On first run (no feeds configured), you'll see a welcome screen:
- **[a]** Add a feed URL manually
- **[i]** Import from OPML file (searches `~/Downloads/` for `.opml` files)

## Command Line Options

```bash
news [OPTIONS] [COMMAND]
```

### Options
- `-c, --config <FILE>` - Path to configuration file
- `-d, --db-path <FILE>` - Path to database file
- `-t, --theme <THEME>` - Theme to use (catppuccin-mocha, claude-code)
- `-h, --help` - Print help

### Commands
- `reset-db` - Reset the database
- `export-feeds` - Export feeds to OPML format
- `import-feeds <FILE>` - Import feeds from OPML file
- `cleanup --days <N>` - Delete posts older than N days
- `info` - Show configuration paths and statistics
- `list-feeds` - List all configured feeds

## Configuration

### File Locations
- **Config file:** `~/.config/news/config.toml`
- **Database:** `~/.local/share/news/news_feed.db`

### Example config.toml
```toml
[app]
theme = "catppuccin-mocha"  # or "claude-code"
startup_cleanup = false

[ui]
show_ascii_banner = true
default_tab = "fresh"

[feeds]
urls = []

[[feeds.sources]]
url = "https://dev.to/rss"
category = "Technology"

[[feeds.sources]]
url = "https://nesslabs.com/feed"
category = "Productivity"
```

## Uninstall

```bash
./uninstall.sh
```

This removes the binary from `/usr/local/bin` but preserves your configuration and database files.

## License

MIT
