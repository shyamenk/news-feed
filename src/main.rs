use std::{error::Error, io, time::Duration};
use crossterm::{
    event::{self, Event, KeyCode, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use futures::StreamExt;

mod app;
mod ascii_art;
mod categories;
mod cli;
mod config;
mod db;
mod input;
mod navigation;
mod rss;
mod stats;
mod tabs;
mod theme;
mod ui;

use app::{App, ConfirmAction, InputMode};
use cli::{Cli, Commands};
use navigation::{FocusPane, NavNode, SidebarSection};
use std::sync::{Arc, Mutex};

fn import_opml_content(content: &str, db: &Arc<Mutex<db::Database>>) -> usize {
    let mut count = 0;
    let mut current_category = "General".to_string();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<outline") && !trimmed.contains("xmlUrl") {
            if let Some(start) = trimmed.find("text=\"") {
                let rest = &trimmed[start + 6..];
                if let Some(end) = rest.find('"') {
                    current_category = rest[..end]
                        .replace("&amp;", "&")
                        .replace("&lt;", "<")
                        .replace("&gt;", ">")
                        .replace("&quot;", "\"")
                        .to_string();
                }
            }
        }

        if trimmed.contains("xmlUrl=") {
            if let Some(start) = trimmed.find("xmlUrl=\"") {
                let rest = &trimmed[start + 8..];
                if let Some(end) = rest.find('"') {
                    let url = &rest[..end];
                    if let Ok(db) = db.lock() {
                        if db.add_feed_with_category(url, &current_category).is_ok() {
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    count
}

async fn fetch_feeds_for_node(
    db: Arc<Mutex<db::Database>>,
    node: NavNode,
    tx: tokio::sync::mpsc::Sender<NavNode>,
) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("news-feed-tui/0.1")
        .build()
        .unwrap();

    let feeds_list = {
        let db = db.lock().unwrap();
        match &node {
            NavNode::SmartView(_) => db.get_feeds().unwrap_or_default(),
            NavNode::Category(cat) => db.get_feeds_by_category(cat).unwrap_or_default(),
        }
    };

    for feed_meta in feeds_list {
        match rss::fetch_feed(&client, &feed_meta.url).await {
            Ok(feed_data) => {
                let db = db.lock().unwrap();
                for entry in feed_data.entries {
                    let title = entry.title.map(|t| t.content).unwrap_or_default();
                    let url = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();

                    let mut content = entry.content.and_then(|c| c.body).unwrap_or_default();
                    if content.trim().is_empty() {
                        content = entry.summary.map(|s| s.content).unwrap_or_default();
                    }

                    let pub_date = entry.published.or(entry.updated);
                    let _ = db.insert_post(feed_meta.id, &title, &url, Some(&content), pub_date);
                }
            }
            Err(_) => {}
        }
    }

    let _ = tx.send(node).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse_args();

    if let Some(ref command) = cli.command {
        return handle_command(command.clone(), &cli).await;
    }

    let config_path = cli.get_config_path();
    let config = config::load_config_from_path(&config_path).unwrap_or_else(|e| {
        eprintln!("Error loading config: {}. Using default.", e);
        config::Config {
            app: config::AppConfig::default(),
            ui: config::UiConfig::default(),
            feeds: config::FeedsConfig::default(),
        }
    });

    let db_path = cli.get_db_path();
    let db = db::Database::init_with_path(&db_path)?;
    let _ = db.ensure_categories_table();

    if !config.feeds.sources.is_empty() {
        for source in &config.feeds.sources {
            for url in source.get_urls() {
                let _ = db.add_feed_with_category(&url, &source.category);
            }
        }
    } else {
        for url in &config.feeds.urls {
            let _ = db.add_feed(url);
        }
    }

    let mut app = App::new(db);
    let db_clone = app.db.clone();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<NavNode>(10);

    if !app.feeds.is_empty() {
        let db_for_fetch = db_clone.clone();
        let tx_clone = tx.clone();
        let initial_node = app.active_node.clone();
        tokio::spawn(async move {
            fetch_feeds_for_node(db_for_fetch, initial_node, tx_clone).await;
        });
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut reader = EventStream::new();
    let theme_name = cli.theme.clone().unwrap_or_else(|| config.app.theme.clone());

    loop {
        terminal.draw(|f| ui::ui(f, &mut app, &theme_name))?;

        tokio::select! {
            Some(fetched_node) = rx.recv() => {
                app.sidebar.mark_fetched(fetched_node.clone());
                if app.active_node == fetched_node {
                    app.reload_posts_for_active_node();
                }
                app.refresh_sidebar();
                app.is_loading = false;
                app.message = Some("Feeds updated".to_string());
            }
            Some(Ok(event)) = reader.next() => {
                match event {
                    Event::Key(key) => {
                        if key.kind == event::KeyEventKind::Press {
                            if app.message.is_some() && !matches!(app.input_mode, InputMode::Confirming(_)) {
                                app.message = None;
                                continue;
                            }

                            match &app.input_mode {
                                InputMode::Welcome => {
                                    handle_welcome_input(&mut app, key.code, &tx, &db_clone);
                                }
                                InputMode::Help => {
                                    app.input_mode = InputMode::Normal;
                                }
                                InputMode::AddingFeed => {
                                    handle_adding_feed_input(&mut app, key.code);
                                }
                                InputMode::AddingCategory => {
                                    handle_adding_category_input(&mut app, key.code);
                                }
                                InputMode::SelectingCategory => {
                                    handle_selecting_category_input(&mut app, key.code);
                                }
                                InputMode::Confirming(action) => {
                                    let action_clone = action.clone();
                                    handle_confirm_input(&mut app, key.code, action_clone);
                                }
                                InputMode::EditingCategoryFeeds(cat) => {
                                    let cat_clone = cat.clone();
                                    handle_editing_category_feeds_input(&mut app, key.code, &cat_clone);
                                }
                                InputMode::Normal => {
                                    handle_normal_input(&mut app, key.code, &tx, &db_clone);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if app.exit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_welcome_input(
    app: &mut App,
    key: KeyCode,
    tx: &tokio::sync::mpsc::Sender<NavNode>,
    db: &Arc<Mutex<db::Database>>,
) {
    match key {
        KeyCode::Char('q') => app.exit = true,
        KeyCode::Char('a') => {
            app.input_mode = InputMode::AddingFeed;
        }
        KeyCode::Char('i') => {
            let home = std::env::var("HOME").unwrap_or_default();
            let opml_paths = vec![
                format!("{}/Downloads/feeds_organized.opml", home),
                format!("{}/Downloads/feeds.opml", home),
                format!("{}/feeds.opml", home),
            ];

            let mut imported = 0;
            for path in opml_paths {
                if std::path::Path::new(&path).exists() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        imported += import_opml_content(&content, &app.db);
                    }
                    break;
                }
            }

            if imported > 0 {
                app.reload_feeds();
                app.refresh_sidebar();
                app.is_loading = true;
                app.input_mode = InputMode::Normal;
                app.message = Some(format!("Imported {} feeds!", imported));

                let db_clone = db.clone();
                let tx_clone = tx.clone();
                let node = app.active_node.clone();
                tokio::spawn(async move {
                    fetch_feeds_for_node(db_clone, node, tx_clone).await;
                });
            } else {
                app.message = Some("No OPML file found in ~/Downloads".to_string());
            }
        }
        _ => {}
    }
}

fn handle_adding_feed_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => app.text_input.insert_char(c),
        KeyCode::Backspace => app.text_input.delete_char(),
        KeyCode::Left => app.text_input.move_cursor_left(),
        KeyCode::Right => app.text_input.move_cursor_right(),
        KeyCode::Enter => {
            if !app.text_input.value.is_empty() {
                app.pending_feed_url = Some(app.text_input.value.clone());
                app.text_input.clear();
                app.input_mode = InputMode::SelectingCategory;
            }
        }
        KeyCode::Esc => {
            app.text_input.clear();
            app.pending_feed_url = None;
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn handle_adding_category_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => app.text_input.insert_char(c),
        KeyCode::Backspace => app.text_input.delete_char(),
        KeyCode::Left => app.text_input.move_cursor_left(),
        KeyCode::Right => app.text_input.move_cursor_right(),
        KeyCode::Enter => {
            if !app.text_input.value.is_empty() {
                app.add_category(&app.text_input.value.clone());
                app.text_input.clear();
                app.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Esc => {
            app.text_input.clear();
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn handle_selecting_category_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Down | KeyCode::Char('j') => {
            if app.sidebar.category_index < app.sidebar.categories.len().saturating_sub(1) {
                app.sidebar.category_index += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.sidebar.category_index > 0 {
                app.sidebar.category_index -= 1;
            }
        }
        KeyCode::Enter => {
            if let Some(url) = app.pending_feed_url.take() {
                let category = app.get_selected_category();
                app.add_feed(&url, &category);
                app.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Esc => {
            app.pending_feed_url = None;
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

fn handle_confirm_input(app: &mut App, key: KeyCode, action: ConfirmAction) {
    match key {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match action {
                ConfirmAction::DeletePost(id) => {
                    if app.db.lock().unwrap().delete_post(id).is_ok() {
                        app.posts.retain(|p| p.id != id);
                        if app.selected_index >= app.posts.len() && !app.posts.is_empty() {
                            app.selected_index = app.posts.len() - 1;
                        }
                        app.refresh_sidebar();
                        app.message = Some("Post deleted".to_string());
                    }
                }
                ConfirmAction::DeleteFeed(id) => {
                    if app.db.lock().unwrap().delete_feed(id).is_ok() {
                        app.reload_feeds();
                        app.refresh_sidebar();
                        app.reload_posts_for_active_node();
                        app.message = Some("Feed deleted".to_string());
                    }
                }
                ConfirmAction::DeleteCategory(name) => {
                    if app.db.lock().unwrap().delete_category(&name).is_ok() {
                        app.refresh_sidebar();
                        app.reload_posts_for_active_node();
                        app.message = Some(format!("Category '{}' deleted", name));
                    }
                }
            }
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.message = None;
        }
        _ => {}
    }
}

fn handle_normal_input(
    app: &mut App,
    key: KeyCode,
    tx: &tokio::sync::mpsc::Sender<NavNode>,
    db: &Arc<Mutex<db::Database>>,
) {
    match key {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.exit = true,
        KeyCode::Char('?') => app.input_mode = InputMode::Help,
        KeyCode::Char('h') | KeyCode::Left => app.focus_left(),
        KeyCode::Char('l') | KeyCode::Right => app.focus_right(),
        KeyCode::Tab => {
            app.focus = match app.focus {
                FocusPane::Sidebar => FocusPane::Posts,
                FocusPane::Posts => FocusPane::Sidebar,
                FocusPane::Article => FocusPane::Sidebar,
            };
        }
        KeyCode::BackTab => {
            app.focus = match app.focus {
                FocusPane::Sidebar => FocusPane::Posts,
                FocusPane::Posts => FocusPane::Sidebar,
                FocusPane::Article => FocusPane::Posts,
            };
        }
        _ => match app.focus {
            FocusPane::Sidebar => handle_sidebar_input(app, key),
            FocusPane::Posts => handle_posts_input(app, key, tx, db),
            FocusPane::Article => handle_article_input(app, key),
        },
    }
}

fn handle_sidebar_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Down | KeyCode::Char('j') => app.sidebar.next(),
        KeyCode::Up | KeyCode::Char('k') => app.sidebar.previous(),
        KeyCode::Enter => app.select_sidebar_item(),
        KeyCode::Char('a') | KeyCode::Char('+') => {
            // Always add feed - will prompt for category selection
            app.input_mode = InputMode::AddingFeed;
        }
        KeyCode::Char('n') => {
            // Add new category
            app.input_mode = InputMode::AddingCategory;
        }
        KeyCode::Char('e') => {
            // Edit category feeds
            if let SidebarSection::Categories = app.sidebar.section {
                if let Some(cat) = app.sidebar.categories.get(app.sidebar.category_index).cloned() {
                    app.load_category_feeds(&cat);
                    app.input_mode = InputMode::EditingCategoryFeeds(cat);
                }
            }
        }
        KeyCode::Char('d') => {
            if let SidebarSection::Categories = app.sidebar.section {
                if let Some(cat) = app.sidebar.categories.get(app.sidebar.category_index).cloned() {
                    if cat == "General" {
                        app.message = Some("Cannot delete 'General' category".to_string());
                    } else {
                        app.input_mode = InputMode::Confirming(ConfirmAction::DeleteCategory(cat));
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_editing_category_feeds_input(app: &mut App, key: KeyCode, category: &str) {
    match key {
        KeyCode::Down | KeyCode::Char('j') => app.next_category_feed(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_category_feed(),
        KeyCode::Char('d') => {
            app.delete_category_feed();
            if app.category_feeds.is_empty() {
                app.input_mode = InputMode::Normal;
            }
        }
        KeyCode::Char('a') | KeyCode::Char('+') => {
            // Add feed to this category - store the category and switch to add feed mode
            app.pending_feed_url = None;
            app.sidebar.category_index = app
                .sidebar
                .categories
                .iter()
                .position(|c| c == category)
                .unwrap_or(0);
            app.input_mode = InputMode::AddingFeed;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.reload_posts_for_active_node();
        }
        _ => {}
    }
}

fn handle_posts_input(
    app: &mut App,
    key: KeyCode,
    tx: &tokio::sync::mpsc::Sender<NavNode>,
    db: &Arc<Mutex<db::Database>>,
) {
    match key {
        KeyCode::Down | KeyCode::Char('j') => app.next_post(),
        KeyCode::Up | KeyCode::Char('k') => app.previous_post(),
        KeyCode::Enter => app.open_article(),
        KeyCode::Char('b') => app.toggle_bookmark(),
        KeyCode::Char('l') => app.toggle_read_later(),
        KeyCode::Char('a') => app.toggle_archived(),
        KeyCode::Char('m') => app.toggle_read(),
        KeyCode::Char('u') => app.toggle_show_read(),
        KeyCode::Char('d') => {
            if let Some(post) = app.posts.get(app.selected_index) {
                app.input_mode = InputMode::Confirming(ConfirmAction::DeletePost(post.id));
            }
        }
        KeyCode::Char('o') => {
            if let Some(post) = app.posts.get(app.selected_index) {
                let _ = open::that(&post.url);
                app.message = Some("Opened in browser".to_string());
            }
        }
        KeyCode::Char('y') => app.copy_url_to_clipboard(),
        KeyCode::Char('r') => {
            if !app.is_loading {
                app.is_loading = true;
                let db_clone = db.clone();
                let tx_clone = tx.clone();
                let node = app.active_node.clone();
                tokio::spawn(async move {
                    fetch_feeds_for_node(db_clone, node, tx_clone).await;
                });
            }
        }
        KeyCode::Char('+') => {
            app.input_mode = InputMode::AddingFeed;
        }
        _ => {}
    }
}

fn handle_article_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => app.close_article(),
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_offset = app.scroll_offset.saturating_add(1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_offset = app.scroll_offset.saturating_sub(1);
        }
        KeyCode::PageDown => {
            app.scroll_offset = app.scroll_offset.saturating_add(10);
        }
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::Char('b') => app.toggle_bookmark(),
        KeyCode::Char('l') => app.toggle_read_later(),
        KeyCode::Char('a') => app.toggle_archived(),
        KeyCode::Char('o') => {
            if let Some(post) = app.posts.get(app.selected_index) {
                let _ = open::that(&post.url);
                app.message = Some("Opened in browser".to_string());
            }
        }
        KeyCode::Char('y') => app.copy_url_to_clipboard(),
        _ => {}
    }
}

async fn handle_command(command: Commands, cli: &Cli) -> Result<(), Box<dyn Error>> {
    match command {
        Commands::ResetDb { yes } => {
            let db_path = cli.get_db_path();

            if !yes {
                println!("This will delete all feeds and posts from the database.");
                println!("Database location: {}", db_path.display());
                print!("Are you sure? (y/N): ");
                io::Write::flush(&mut io::stdout())?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let db = db::Database::init_with_path(&db_path)?;
            db.reset()?;
            println!("Database reset successfully.");
        }

        Commands::ExportFeeds { output } => {
            let db_path = cli.get_db_path();
            let db = db::Database::init_with_path(&db_path)?;
            let feeds = db.get_feeds()?;

            let mut opml = String::from(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head>
    <title>News Feed Subscriptions</title>
  </head>
  <body>
"#,
            );

            for feed in feeds {
                let title = feed.title.as_deref().unwrap_or("Untitled");
                opml.push_str(&format!(
                    r#"    <outline type="rss" text="{}" xmlUrl="{}" category="{}"/>
"#,
                    title, feed.url, feed.category
                ));
            }

            opml.push_str("  </body>\n</opml>\n");

            if let Some(output_path) = output {
                std::fs::write(&output_path, opml)?;
                println!("Feeds exported to: {}", output_path.display());
            } else {
                print!("{}", opml);
            }
        }

        Commands::ImportFeeds { input } => {
            println!("Reading from: {}", input.display());

            let content = std::fs::read_to_string(&input)?;
            let db_path = cli.get_db_path();
            let db = db::Database::init_with_path(&db_path)?;

            let mut count = 0;
            for line in content.lines() {
                if line.contains("xmlUrl=") {
                    if let Some(start) = line.find("xmlUrl=\"") {
                        let rest = &line[start + 8..];
                        if let Some(end) = rest.find('"') {
                            let url = &rest[..end];
                            let category = if let Some(cat_start) = line.find("category=\"") {
                                let cat_rest = &line[cat_start + 10..];
                                if let Some(cat_end) = cat_rest.find('"') {
                                    &cat_rest[..cat_end]
                                } else {
                                    "General"
                                }
                            } else {
                                "General"
                            };

                            match db.add_feed_with_category(url, category) {
                                Ok(_) => count += 1,
                                Err(e) => eprintln!("Failed to add {}: {}", url, e),
                            }
                        }
                    }
                }
            }

            println!("Imported {} feeds.", count);
        }

        Commands::Cleanup { days, yes } => {
            let db_path = cli.get_db_path();

            if !yes {
                println!(
                    "This will delete all posts older than {} days (except bookmarked).",
                    days
                );
                print!("Are you sure? (y/N): ");
                io::Write::flush(&mut io::stdout())?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled.");
                    return Ok(());
                }
            }

            let db = db::Database::init_with_path(&db_path)?;
            let count = db.cleanup_old_posts(days)?;
            println!("Deleted {} old posts.", count);
        }

        Commands::Info => {
            let config_path = cli.get_config_path();
            let db_path = cli.get_db_path();

            println!("News Feed Reader v0.1.0");
            println!();
            println!("Configuration:");
            println!("  Config file: {}", config_path.display());
            println!("  Database:    {}", db_path.display());
            println!();

            if db_path.exists() {
                let db = db::Database::init_with_path(&db_path)?;
                let total_posts = db.get_total_posts_count()?;
                let total_feeds = db.get_total_feeds_count()?;

                println!("Statistics:");
                println!("  Total feeds: {}", total_feeds);
                println!("  Total posts: {}", total_posts);
            } else {
                println!("Database does not exist yet. Run 'news' to create it.");
            }
        }

        Commands::ListFeeds => {
            let db_path = cli.get_db_path();

            if !db_path.exists() {
                println!("No database found. Run 'news' first to create it.");
                return Ok(());
            }

            let db = db::Database::init_with_path(&db_path)?;
            let feeds = db.get_feeds()?;

            if feeds.is_empty() {
                println!("No feeds configured yet.");
            } else {
                println!("Configured feeds ({}):", feeds.len());
                println!();

                for feed in feeds {
                    let title = feed.title.as_deref().unwrap_or("(No title)");
                    println!("  [{}] {} - {}", feed.category, title, feed.url);
                }
            }
        }
    }

    Ok(())
}
