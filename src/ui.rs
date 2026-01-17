use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::app::{App, InputMode, ViewContext};
use crate::theme::{Theme, ThemeVariant};
use crate::tabs::Tab;
use crate::ascii_art::NEWS_BANNER;
use html2text;

pub fn ui(f: &mut Frame, app: &mut App, theme_name: &str) {
    let theme_variant = ThemeVariant::from_str(theme_name);
    let theme = theme_variant.get_theme();

    let size = f.area();
    let block = Block::default().style(Style::default().bg(theme.base()));
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_tab_bar(f, app, chunks[0], &*theme);

    match (&app.input_mode, app.tabs.get_active(), &app.view_context) {
        // Article view takes priority
        (InputMode::Normal, _, ViewContext::Article) => draw_article(f, app, chunks[1], &*theme),
        // Dashboard
        (InputMode::Normal, Tab::Dashboard, _) => draw_dashboard(f, app, chunks[1], &*theme, app.is_loading),
        // Feed Manager
        (InputMode::Normal, Tab::FeedManager, _) => draw_feed_manager(f, app, chunks[1], &*theme),
        (InputMode::AddingFeed, Tab::FeedManager, _) => draw_feed_manager_adding(f, app, chunks[1], &*theme),
        // Category selection for new feed
        (InputMode::SelectingCategoryForFeed, _, _) => draw_category_selector(f, app, chunks[1], &*theme, "Select Category for Feed"),
        // Category tab views
        (InputMode::SelectingCategoryForView, Tab::Category, _) => draw_category_dropdown(f, app, chunks[1], &*theme),
        (InputMode::AddingCategory, Tab::Category, _) => draw_add_category(f, app, chunks[1], &*theme),
        (InputMode::Normal, Tab::Category, ViewContext::List) => {
            if app.category_view.active_category.is_some() {
                draw_posts_list(f, app, chunks[1], &*theme);
            } else {
                draw_category_select_prompt(f, app, chunks[1], &*theme);
            }
        }
        // Default list view
        (_, _, ViewContext::List) => {
            if app.is_loading {
                draw_loading(f, chunks[1], &*theme);
            } else {
                draw_posts_list(f, app, chunks[1], &*theme);
            }
        }
        _ => {}
    }

    draw_status_bar(f, app, chunks[2], &*theme);
}

fn draw_tab_bar(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let tab_titles: Vec<String> = app.tabs.tabs.iter().map(|t| {
        format!("{}{}", t.icon(), t.title())
    }).collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" 󰑫 News Reader ")
            .title_style(Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD)))
        .select(app.tabs.active_index)
        .style(Style::default().fg(theme.subtext()))
        .highlight_style(Style::default()
            .fg(theme.accent_secondary())
            .add_modifier(Modifier::BOLD));

    f.render_widget(tabs, area);
}

fn draw_loading(f: &mut Frame, area: Rect, theme: &dyn Theme) {
    let art = vec![
        "      )  (  )  ",
        "     (   )  (  ",
        "   . '  . ' .  ",
        "   '  . ' . '  ",
        "  ___________",
        " |           | ",
        " |  Coffee   | ",
        " |   & News  | ",
        " |___________| ",
        "  \\_________/  ",
        "",
        " Loading feeds...",
    ];

    let text_art: Vec<Line> = art
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))))
        .collect();

    let paragraph = Paragraph::new(text_art)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    let area = centered_rect(50, 50, area);
    f.render_widget(paragraph, area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme, is_loading: bool) {
    let stats = &app.cached_stats;

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(area);

    let banner = Paragraph::new(NEWS_BANNER)
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD));
    f.render_widget(banner, main_chunks[0]);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(60),
        ])
        .split(main_chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(body_chunks[0]);

    let quote_block = Paragraph::new(Line::from(vec![
        Span::styled(app.dashboard_quote, Style::default().fg(theme.text()).add_modifier(Modifier::ITALIC)),
    ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(" 💭 Identity ")
            .title_style(Style::default().fg(theme.accent_secondary())))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(quote_block, left_chunks[0]);

    let usage_lines = vec![
        Line::from(vec![
            Span::styled("[1-6]", Style::default().fg(theme.warning())),
            Span::styled(" Navigate Tabs", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(theme.warning())),
            Span::styled(" Next Tab", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[j/k]", Style::default().fg(theme.warning())),
            Span::styled(" Up/Down", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(theme.warning())),
            Span::styled(" Select/Open", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[B]", Style::default().fg(theme.warning())),
            Span::styled(" Favourite", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[L]", Style::default().fg(theme.warning())),
            Span::styled(" Read Later", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[A]", Style::default().fg(theme.warning())),
            Span::styled(" Archive", Style::default().fg(theme.text())),
        ]),
        Line::from(vec![
            Span::styled("[Q]", Style::default().fg(theme.warning())),
            Span::styled(" Quit", Style::default().fg(theme.text())),
        ]),
    ];

    let usage_block = Paragraph::new(usage_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" ⌨ Usage ")
            .title_style(Style::default().fg(theme.accent_primary())))
        .style(Style::default().fg(theme.text()));
    f.render_widget(usage_block, left_chunks[1]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(body_chunks[1]);

    let status_text = if is_loading { " (Loading...)" } else { "" };
    let total_line = Line::from(vec![
        Span::styled("󰈙 ", Style::default().fg(theme.accent_primary())),
        Span::styled(format!("{}", stats.total_posts), Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
        Span::styled(" Articles  │  ", Style::default().fg(theme.text())),
        Span::styled("󰑫 ", Style::default().fg(theme.accent_primary())),
        Span::styled(format!("{}", stats.feeds_count), Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
        Span::styled(" Feeds  │  ", Style::default().fg(theme.text())),
        Span::styled("󰻞 ", Style::default().fg(theme.accent_primary())),
        Span::styled(format!("{}", stats.categories.len()), Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
        Span::styled(" Categories", Style::default().fg(theme.text())),
        Span::styled(status_text, Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::ITALIC)),
    ]);
    let total_widget = Paragraph::new(total_line)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary())))
        .alignment(Alignment::Center);
    f.render_widget(total_widget, right_chunks[0]);

    let read_ratio = stats.reading_progress();
    let read_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(format!(" 📖 Read: {}/{} ", stats.read_posts, stats.total_posts)))
        .gauge_style(Style::default().fg(theme.success()))
        .ratio(read_ratio)
        .label(format!("{}%", (read_ratio * 100.0) as u8));
    f.render_widget(read_gauge, right_chunks[1]);

    let fav_ratio = if stats.total_posts > 0 {
        stats.saved_posts as f64 / stats.total_posts as f64
    } else { 0.0 };
    let fav_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(format!(" 󰃀 Favourites: {} ", stats.saved_posts)))
        .gauge_style(Style::default().fg(theme.success()))
        .ratio(fav_ratio);
    f.render_widget(fav_gauge, right_chunks[2]);

    let archived_ratio = if stats.total_posts > 0 {
        stats.archived_posts as f64 / stats.total_posts as f64
    } else { 0.0 };
    let archived_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(format!(" 󰆧 Archived: {} ", stats.archived_posts)))
        .gauge_style(Style::default().fg(theme.success()))
        .ratio(archived_ratio);
    f.render_widget(archived_gauge, right_chunks[3]);

    let later_ratio = if stats.total_posts > 0 {
        stats.read_later_posts as f64 / stats.total_posts as f64
    } else { 0.0 };
    let later_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(format!(" 󰃰 Read Later: {} ", stats.read_later_posts)))
        .gauge_style(Style::default().fg(theme.success()))
        .ratio(later_ratio);
    f.render_widget(later_gauge, right_chunks[4]);

    let cat_lines: Vec<Line> = stats.categories.iter()
        .map(|(cat, count)| Line::from(vec![
            Span::styled(format!("  󰻞 {} ", cat), Style::default().fg(theme.accent_secondary())),
            Span::styled(format!("{} posts", count), Style::default().fg(theme.warning())),
        ]))
        .collect();

    let cat_widget = Paragraph::new(cat_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" 󰺯 Categories ")
            .title_style(Style::default().fg(theme.accent_primary())))
        .style(Style::default().fg(theme.text()));
    f.render_widget(cat_widget, right_chunks[5]);
}

fn draw_category_select_prompt(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("No category selected", Style::default().fg(theme.subtext()).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text())),
            Span::styled("[Enter]", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled(" to choose a category", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text())),
            Span::styled("[+]", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled(" to add a new category", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.text())),
            Span::styled("[d]", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled(" to delete selected category", Style::default().fg(theme.text())),
        ]),
    ];

    let available_cats: Vec<Line> = app.category_view.categories.iter()
        .map(|cat| Line::from(Span::styled(format!("  • {}", cat), Style::default().fg(theme.accent_secondary()))))
        .collect();

    let mut all_lines = lines;
    all_lines.push(Line::from(""));
    all_lines.push(Line::from(Span::styled("Available Categories:", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))));
    all_lines.extend(available_cats);

    let widget = Paragraph::new(all_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" 󰻞 Category ")
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)))
        .alignment(Alignment::Center);

    f.render_widget(widget, area);
}

fn draw_category_dropdown(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let items: Vec<ListItem> = app.category_view.categories.iter().enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.category_view.selected_index;
            let style = if is_selected {
                Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            };
            let prefix = if is_selected { ">> " } else { "   " };
            ListItem::new(format!("{}{}", prefix, cat)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .title(" Select Category (j/k navigate, Enter select, Esc cancel) ")
            .title_style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)));

    let area = centered_rect(50, 60, area);
    f.render_widget(list, area);
}

fn draw_add_category(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let content = format!("Category Name: {}_", app.text_input.value);

    let widget = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .title(" Add New Category (Enter confirm, Esc cancel) ")
            .title_style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(theme.text()));

    let area = centered_rect(50, 20, area);
    f.render_widget(widget, area);
}

fn draw_feed_manager(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(5),
        ])
        .split(area);

    if app.feeds.is_empty() {
        let empty = Paragraph::new("No feeds configured. Press [+] to add a feed.")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(" 󰑫 Feeds ")
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.subtext()));
        f.render_widget(empty, chunks[0]);
    } else {
        let items: Vec<ListItem> = app.feeds.iter().enumerate()
            .map(|(i, feed)| {
                let is_selected = i == app.selected_feed_index;
                let prefix = if is_selected { ">> " } else { "   " };
                let style = if is_selected {
                    Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text())
                };
                
                let title_display = feed.title.as_deref().unwrap_or(&feed.url);
                let line = Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(title_display, style),
                    Span::styled(format!(" [{}]", feed.category), Style::default().fg(theme.accent_secondary())),
                ]);
                ListItem::new(line)
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(app.selected_feed_index));

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(format!(" 󰑫 Feeds ({}) ", app.feeds.len()))
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)))
            .highlight_style(Style::default().bg(theme.highlight()));

        f.render_stateful_widget(list, chunks[0], &mut list_state);
    }

    let help_lines = vec![
        Line::from(vec![
            Span::styled("[+/n]", Style::default().fg(theme.warning())),
            Span::styled(" Add Feed  ", Style::default().fg(theme.text())),
            Span::styled("[d]", Style::default().fg(theme.warning())),
            Span::styled(" Delete  ", Style::default().fg(theme.text())),
            Span::styled("[j/k]", Style::default().fg(theme.warning())),
            Span::styled(" Navigate", Style::default().fg(theme.text())),
        ]),
    ];

    let help = Paragraph::new(help_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.subtext()))
            .title(" Help "))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[1]);
}

fn draw_feed_manager_adding(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let content = format!("Feed URL: {}_", app.text_input.value);

    let widget = Paragraph::new(content)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .title(" Add Feed (Enter to continue, Esc cancel) ")
            .title_style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(theme.text()));

    let area = centered_rect(70, 20, area);
    f.render_widget(widget, area);
}

fn draw_category_selector(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme, title: &str) {
    use crate::categories::CATEGORIES;

    let items: Vec<ListItem> = CATEGORIES.iter().enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.category_selector.selected_index;
            let style = if is_selected {
                Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            };
            let prefix = if is_selected { ">> " } else { "   " };
            ListItem::new(format!("{}{}", prefix, cat)).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .title(format!(" {} (j/k navigate, Enter select) ", title))
            .title_style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)));

    let area = centered_rect(50, 60, area);
    f.render_widget(list, area);
}

fn draw_posts_list(f: &mut Frame, app: &mut App, area: Rect, theme: &dyn Theme) {
    if app.posts.is_empty() {
        let msg = match app.tabs.get_active() {
            Tab::Category => {
                if let Some(ref cat) = app.category_view.active_category {
                    format!("No posts in '{}' category", cat)
                } else {
                    "Select a category first".to_string()
                }
            }
            Tab::Favourite => "No favourites yet. Press [B] on a post to add.".to_string(),
            Tab::ReadLater => "No read later items. Press [L] on a post to add.".to_string(),
            Tab::Archived => "No archived posts. Press [A] on a post to archive.".to_string(),
            _ => "No posts available".to_string(),
        };

        let empty = Paragraph::new(msg)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(format!(" {} ", app.tabs.get_active().title())))
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.subtext()));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app.posts
        .iter()
        .map(|post| {
            let title = if post.is_read {
                Span::styled(&post.title, Style::default().fg(theme.overlay()))
            } else {
                Span::styled(&post.title, Style::default().fg(theme.text()).add_modifier(Modifier::BOLD))
            };

            let feed = Span::styled(
                format!(" [{}]", post.feed_title.as_deref().unwrap_or("?")),
                Style::default().fg(theme.warning())
            );

            let mut badges = vec![];
            if post.is_bookmarked {
                badges.push(Span::styled(" [FAV]", Style::default().fg(theme.accent_secondary())));
            }
            if post.is_archived {
                badges.push(Span::styled(" [ARC]", Style::default().fg(theme.subtext())));
            }
            if post.is_read_later {
                badges.push(Span::styled(" [LATER]", Style::default().fg(theme.success())));
            }

            let mut line_parts = vec![title, feed];
            line_parts.extend(badges);

            ListItem::new(Line::from(line_parts))
        })
        .collect();

    let tab_title = match app.tabs.get_active() {
        Tab::Category => {
            if let Some(ref cat) = app.category_view.active_category {
                format!(" {} ({}) ", cat, app.posts.len())
            } else {
                " Category ".to_string()
            }
        }
        _ => format!(" {} ({}) ", app.tabs.get_active().title(), app.posts.len()),
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(tab_title)
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)))
        .highlight_style(Style::default().bg(theme.highlight()).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));
    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_article(f: &mut Frame, app: &mut App, area: Rect, theme: &dyn Theme) {
    if let Some(post) = app.posts.get(app.selected_index) {
        let content_raw = post.content.clone().unwrap_or_default();
        let display_content = if content_raw.trim().is_empty() {
            format!("No content available for this post.\n\nLink: {}", post.url)
        } else {
            content_raw
        };

        let text_content = html2text::from_read(display_content.as_bytes(), area.width as usize)
            .unwrap_or_else(|_| format!("Error parsing content.\n\nLink: {}", post.url));

        let mut title_badges = vec![];
        if post.is_bookmarked {
            title_badges.push("[FAV]");
        }
        if post.is_archived {
            title_badges.push("[ARC]");
        }
        if post.is_read_later {
            title_badges.push("[LATER]");
        }

        let title_text = if title_badges.is_empty() {
            post.title.clone()
        } else {
            format!("{} {}", post.title, title_badges.join(" "))
        };

        let paragraph = Paragraph::new(text_content)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(title_text.as_str())
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(theme.text()))
            .wrap(Wrap { trim: true })
            .scroll((app.scroll_offset, 0));

        f.render_widget(paragraph, area);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let mode = if let Some(msg) = &app.message {
        format!(" {} (Press any key)", msg)
    } else {
        match (&app.input_mode, &app.view_context, app.tabs.get_active()) {
            (InputMode::AddingFeed, _, _) => " Type URL │ [Enter] Continue │ [Esc] Cancel".to_string(),
            (InputMode::SelectingCategoryForFeed, _, _) => " [j/k] Navigate │ [Enter] Select │ [Esc] Cancel".to_string(),
            (InputMode::SelectingCategoryForView, _, _) => " [j/k] Navigate │ [Enter] Select │ [Esc] Cancel".to_string(),
            (InputMode::AddingCategory, _, _) => " Type name │ [Enter] Add │ [Esc] Cancel".to_string(),
            (InputMode::Normal, ViewContext::Article, _) => " [Esc] Back │ [j/k] Scroll │ [B] Fav │ [A] Archive │ [L] Later │ [Q] Quit".to_string(),
            (InputMode::Normal, ViewContext::List, Tab::Dashboard) => " [1-6] Nav │ [Tab] Next │ [Q] Quit".to_string(),
            (InputMode::Normal, ViewContext::List, Tab::FeedManager) => " [+] Add │ [d] Delete │ [j/k] Nav │ [Q] Quit".to_string(),
            (InputMode::Normal, ViewContext::List, Tab::Category) => {
                if app.category_view.active_category.is_some() {
                    " [Enter] Read │ [j/k] Nav │ [c] Change Cat │ [B] Fav │ [Q] Quit".to_string()
                } else {
                    " [Enter] Select │ [+] Add Category │ [d] Delete │ [Q] Quit".to_string()
                }
            }
            (InputMode::Normal, ViewContext::List, Tab::Favourite) |
            (InputMode::Normal, ViewContext::List, Tab::ReadLater) |
            (InputMode::Normal, ViewContext::List, Tab::Archived) => {
                " [Enter] Read │ [j/k] Nav │ [d] Delete │ [B] Fav │ [A] Arc │ [L] Later │ [Q] Quit".to_string()
            }
        }
    };

    let style = if app.message.is_some() {
        Style::default().fg(theme.base()).bg(theme.warning()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text()).bg(theme.mantle())
    };

    let status = Paragraph::new(mode).style(style);
    f.render_widget(status, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
