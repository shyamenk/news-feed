use std::{error::Error, io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use futures::StreamExt;

mod app;
mod ascii_art;
mod categories;
mod config;
mod db;
mod input;
mod rss;
mod stats;
mod tabs;
mod theme;
mod ui;

use app::{App, InputMode, ViewContext};
use tabs::Tab;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::load_config().unwrap_or_else(|e| {
        eprintln!("Error loading config: {}. Using default.", e);
        config::Config {
            app: config::AppConfig::default(),
            ui: config::UiConfig::default(),
            feeds: config::FeedsConfig::default(),
        }
    });

    let db = db::Database::init()?;
    let _ = db.ensure_categories_table();

    if !config.feeds.sources.is_empty() {
        for source in &config.feeds.sources {
            let _ = db.add_feed_with_category(&source.url, &source.category);
        }
    } else {
        for url in &config.feeds.urls {
            let _ = db.add_feed(url);
        }
    }

    let mut app = App::new(db);
    let db_clone = app.db.clone();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("news-feed-tui/0.1")
            .build()
            .unwrap();

        let mut first_run = true;

        loop {
            if !first_run {
                tokio::time::sleep(Duration::from_secs(60 * 15)).await;
            }
            first_run = false;

            let mut updated = false;
            let feeds_list = {
                let db = db_clone.lock().unwrap();
                db.get_feeds().unwrap_or_default()
            };

            for feed_meta in feeds_list {
                match rss::fetch_feed(&client, &feed_meta.url).await {
                    Ok(feed_data) => {
                        let db = db_clone.lock().unwrap();
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
                        updated = true;
                    }
                    Err(_e) => {}
                }
            }

            if updated {
                let _ = tx.send(()).await;
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut reader = EventStream::new();
    let theme_name = config.app.theme.clone();

    loop {
        terminal.draw(|f| ui::ui(f, &mut app, &theme_name))?;

        tokio::select! {
            Some(()) = rx.recv() => {
                app.reload_posts_for_current_tab();
                app.refresh_cached_stats();
                app.refresh_categories();
                app.is_loading = false;
                app.message = Some("Feeds updated".to_string());
            }
            Some(Ok(event)) = reader.next() => {
                match event {
                    Event::Key(key) => {
                        if key.kind == event::KeyEventKind::Press {
                            if app.message.is_some() {
                                app.message = None;
                                continue;
                            }

                            match &app.input_mode {
                                InputMode::AddingFeed => {
                                    match key.code {
                                        KeyCode::Char(c) => app.text_input.insert_char(c),
                                        KeyCode::Backspace => app.text_input.delete_char(),
                                        KeyCode::Enter => {
                                            if !app.text_input.value.is_empty() {
                                                app.input_mode = InputMode::SelectingCategoryForFeed;
                                            }
                                        }
                                        KeyCode::Esc => {
                                            app.text_input.clear();
                                            app.input_mode = InputMode::Normal;
                                        }
                                        _ => {}
                                    }
                                }
                                InputMode::SelectingCategoryForFeed => {
                                    match key.code {
                                        KeyCode::Down | KeyCode::Char('j') => app.category_selector.next(),
                                        KeyCode::Up | KeyCode::Char('k') => app.category_selector.previous(),
                                        KeyCode::Enter => {
                                            let url = app.text_input.value.clone();
                                            let category = app.category_selector.get_selected().to_string();
                                            let _ = app.db.lock().unwrap().add_feed_with_category(&url, &category);
                                            app.reload_feeds();
                                            app.refresh_categories();
                                            app.refresh_cached_stats();
                                            app.text_input.clear();
                                            app.input_mode = InputMode::Normal;
                                            app.message = Some(format!("Added feed: {}", url));
                                        }
                                        KeyCode::Esc => {
                                            app.text_input.clear();
                                            app.input_mode = InputMode::Normal;
                                        }
                                        _ => {}
                                    }
                                }
                                InputMode::SelectingCategoryForView => {
                                    match key.code {
                                        KeyCode::Down | KeyCode::Char('j') => app.category_view.next(),
                                        KeyCode::Up | KeyCode::Char('k') => app.category_view.previous(),
                                        KeyCode::Enter => {
                                            app.category_view.select_current();
                                            app.reload_posts_for_current_tab();
                                            app.selected_index = 0;
                                            app.input_mode = InputMode::Normal;
                                        }
                                        KeyCode::Esc => {
                                            app.input_mode = InputMode::Normal;
                                        }
                                        _ => {}
                                    }
                                }
                                InputMode::AddingCategory => {
                                    match key.code {
                                        KeyCode::Char(c) => app.text_input.insert_char(c),
                                        KeyCode::Backspace => app.text_input.delete_char(),
                                        KeyCode::Enter => {
                                            let name = app.text_input.value.clone();
                                            if !name.trim().is_empty() {
                                                app.add_category(&name);
                                            }
                                            app.text_input.clear();
                                            app.input_mode = InputMode::Normal;
                                        }
                                        KeyCode::Esc => {
                                            app.text_input.clear();
                                            app.input_mode = InputMode::Normal;
                                        }
                                        _ => {}
                                    }
                                }

                                InputMode::Normal => {
                                    match key.code {
                                        KeyCode::Char('q') => app.exit = true,
                                        KeyCode::Tab => {
                                            app.tabs.next();
                                            app.reload_posts_for_current_tab();
                                            app.selected_index = 0;
                                        }
                                        KeyCode::BackTab => {
                                            app.tabs.previous();
                                            app.reload_posts_for_current_tab();
                                            app.selected_index = 0;
                                        }
                                        KeyCode::Char('1') => app.switch_to_tab(0),
                                        KeyCode::Char('2') => app.switch_to_tab(1),
                                        KeyCode::Char('3') => app.switch_to_tab(2),
                                        KeyCode::Char('4') => app.switch_to_tab(3),
                                        KeyCode::Char('5') => app.switch_to_tab(4),
                                        KeyCode::Char('6') => app.switch_to_tab(5),
                                        _ => {
                                            match (&app.view_context, app.tabs.get_active()) {
                                                (ViewContext::List, Tab::Dashboard) => {}
                                                (ViewContext::List, Tab::FeedManager) => {
                                                    match key.code {
                                                        KeyCode::Char('+') | KeyCode::Char('n') => {
                                                            app.input_mode = InputMode::AddingFeed;
                                                        }
                                                        KeyCode::Down | KeyCode::Char('j') => app.next_feed(),
                                                        KeyCode::Up | KeyCode::Char('k') => app.previous_feed(),
                                                        KeyCode::Char('d') => app.delete_selected_feed(),
                                                        _ => {}
                                                    }
                                                }
                                                (ViewContext::List, Tab::Category) => {
                                                    if app.category_view.active_category.is_some() {
                                                        match key.code {
                                                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                                                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                                                            KeyCode::Enter => {
                                                                app.open_article();
                                                                app.view_context = ViewContext::Article;
                                                            }
                                                            KeyCode::Char('b') => app.toggle_bookmark(),
                                                            KeyCode::Char('a') => app.toggle_archived(),
                                                            KeyCode::Char('l') => app.toggle_read_later(),
                                                            KeyCode::Char('c') => {
                                                                app.category_view.active_category = None;
                                                                app.posts.clear();
                                                            }
                                                            _ => {}
                                                        }
                                                    } else {
                                                        match key.code {
                                                            KeyCode::Enter => {
                                                                if !app.category_view.categories.is_empty() {
                                                                    app.input_mode = InputMode::SelectingCategoryForView;
                                                                }
                                                            }
                                                            KeyCode::Down | KeyCode::Char('j') => app.category_view.next(),
                                                            KeyCode::Up | KeyCode::Char('k') => app.category_view.previous(),
                                                            KeyCode::Char('+') | KeyCode::Char('n') => {
                                                                app.input_mode = InputMode::AddingCategory;
                                                            }
                                                            KeyCode::Char('d') => app.delete_selected_category(),
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                                (ViewContext::List, Tab::Favourite) | 
                                                (ViewContext::List, Tab::ReadLater) | 
                                                (ViewContext::List, Tab::Archived) => {
                                                    match key.code {
                                                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                                                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                                                        KeyCode::Enter => {
                                                            app.open_article();
                                                            app.view_context = ViewContext::Article;
                                                        }
                                                        KeyCode::Char('b') => app.toggle_bookmark(),
                                                        KeyCode::Char('a') => app.toggle_archived(),
                                                        KeyCode::Char('l') => app.toggle_read_later(),
                                                        KeyCode::Char('d') => app.delete_selected_post(),
                                                        KeyCode::Char('r') => app.reload_posts_for_current_tab(),
                                                        _ => {}
                                                    }
                                                }
                                                (ViewContext::Article, _) => {
                                                    match key.code {
                                                        KeyCode::Esc | KeyCode::Backspace => {
                                                            app.view_context = ViewContext::List;
                                                            app.scroll_offset = 0;
                                                        }
                                                        KeyCode::Down | KeyCode::Char('j') => {
                                                            app.scroll_offset = app.scroll_offset.saturating_add(1);
                                                        }
                                                        KeyCode::Up | KeyCode::Char('k') => {
                                                            app.scroll_offset = app.scroll_offset.saturating_sub(1);
                                                        }
                                                        KeyCode::Char('b') => app.toggle_bookmark(),
                                                        KeyCode::Char('a') => app.toggle_archived(),
                                                        KeyCode::Char('l') => app.toggle_read_later(),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
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
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
