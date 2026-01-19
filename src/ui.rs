use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};
use crate::navigation::{FocusPane, NavNode, SidebarSection, SmartView};
use crate::theme::{Theme, ThemeVariant};

pub fn ui(f: &mut Frame, app: &mut App, theme_name: &str) {
    let theme_variant = ThemeVariant::from_str(theme_name);
    let theme = theme_variant.get_theme();

    let size = f.area();
    let block = Block::default().style(Style::default().bg(theme.base()));
    f.render_widget(block, size);

    match &app.input_mode {
        InputMode::Welcome => {
            draw_welcome(f, app, size, &*theme);
            return;
        }
        InputMode::Help => {
            draw_main_layout(f, app, size, &*theme);
            draw_help_overlay(f, size, &*theme);
            return;
        }
        _ => {}
    }

    draw_main_layout(f, app, size, &*theme);

    match &app.input_mode {
        InputMode::AddingFeed => draw_input_modal(f, app, size, &*theme, "Add Feed URL"),
        InputMode::AddingCategory => draw_input_modal(f, app, size, &*theme, "Add Category"),
        InputMode::SelectingCategory => draw_category_selector(f, app, size, &*theme),
        InputMode::EditingCategoryFeeds(cat) => draw_category_feeds_editor(f, app, size, &*theme, cat),
        InputMode::Confirming(action) => {
            let msg = match action {
                crate::app::ConfirmAction::DeletePost(_) => "Delete this post?",
                crate::app::ConfirmAction::DeleteFeed(_) => "Delete this feed and all its posts?",
                crate::app::ConfirmAction::DeleteCategory(_) => "Delete this category?",
            };
            draw_confirm_modal(f, size, &*theme, msg);
        }
        _ => {}
    }
}

fn draw_main_layout(f: &mut Frame, app: &mut App, area: Rect, theme: &dyn Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, app, chunks[0], theme);

    // In article view, use full screen (no sidebar)
    if matches!(app.focus, FocusPane::Article) {
        draw_article_fullscreen(f, app, chunks[1], theme);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(24), Constraint::Min(0)])
            .split(chunks[1]);

        draw_sidebar(f, app, main_chunks[0], theme);
        draw_posts_list(f, app, main_chunks[1], theme);
    }

    draw_status_bar(f, app, chunks[2], theme);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let loading_indicator = if app.is_loading { " 󰑓 Loading..." } else { "" };

    let title = format!(" 󰑫 News Reader{} ", loading_indicator);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!("[{}]", app.active_node.title()),
            Style::default().fg(theme.accent_secondary()),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.overlay())),
    );

    f.render_widget(header, area);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let is_focused = matches!(app.focus, FocusPane::Sidebar);
    let border_color = if is_focused {
        theme.accent_primary()
    } else {
        theme.overlay()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(if is_focused { " Navigation " } else { " Nav " })
        .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut items: Vec<ListItem> = Vec::new();

    items.push(ListItem::new(Line::from(Span::styled(
        "VIEWS",
        Style::default().fg(theme.subtext()).add_modifier(Modifier::BOLD),
    ))));

    for (i, sv) in app.sidebar.smart_views.iter().enumerate() {
        let is_selected = matches!(app.sidebar.section, SidebarSection::SmartViews)
            && app.sidebar.smart_view_index == i
            && is_focused;

        let count = app.sidebar.get_count(&NavNode::SmartView(sv.clone()));
        let is_active = app.active_node == NavNode::SmartView(sv.clone());

        let prefix = if is_active { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(theme.accent_primary())
        } else {
            Style::default().fg(theme.text())
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(sv.icon(), style),
            Span::styled(format!(" {} ", sv.title()), style),
            Span::styled(format!("({})", count), Style::default().fg(theme.subtext())),
        ])));
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "CATEGORIES",
        Style::default().fg(theme.subtext()).add_modifier(Modifier::BOLD),
    ))));

    for (i, cat) in app.sidebar.categories.iter().enumerate() {
        let is_selected = matches!(app.sidebar.section, SidebarSection::Categories)
            && app.sidebar.category_index == i
            && is_focused;

        let count = app.sidebar.get_count(&NavNode::Category(cat.clone()));
        let is_active = app.active_node == NavNode::Category(cat.clone());

        let prefix = if is_active { "▶ " } else { "  " };
        let style = if is_selected {
            Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(theme.accent_primary())
        } else {
            Style::default().fg(theme.text())
        };

        let display_name = if cat.len() > 12 {
            format!("{}…", &cat[..11])
        } else {
            cat.clone()
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled("󰉋 ", style),
            Span::styled(format!("{} ", display_name), style),
            Span::styled(format!("({})", count), Style::default().fg(theme.subtext())),
        ])));
    }

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_posts_list(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let is_focused = matches!(app.focus, FocusPane::Posts);
    let border_color = if is_focused {
        theme.accent_primary()
    } else {
        theme.overlay()
    };

    let title = format!(
        " {} ({}) ",
        app.active_node.title(),
        app.posts.len()
    );

    if app.posts.is_empty() {
        let empty_msg = match &app.active_node {
            NavNode::SmartView(SmartView::Fresh) => "All caught up! No unread posts.",
            NavNode::SmartView(SmartView::Starred) => "No starred posts yet. Press 'b' to star.",
            NavNode::SmartView(SmartView::ReadLater) => "No posts saved for later. Press 'l' to save.",
            NavNode::SmartView(SmartView::Archived) => "No archived posts.",
            NavNode::Category(_) => "No posts in this category.",
        };

        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(empty_msg, Style::default().fg(theme.subtext()))),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'r' to refresh feeds",
                Style::default().fg(theme.overlay()),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
        );

        f.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .posts
        .iter()
        .enumerate()
        .map(|(i, post)| {
            let is_selected = i == app.selected_index && is_focused;

            let read_indicator = if post.is_read { "○" } else { "●" };
            let read_style = if post.is_read {
                Style::default().fg(theme.overlay())
            } else {
                Style::default().fg(theme.accent_primary())
            };

            let mut badges = String::new();
            if post.is_bookmarked {
                badges.push_str(" ★");
            }
            if post.is_read_later {
                badges.push_str(" 󰃰");
            }
            if post.is_archived {
                badges.push_str(" 󰆧");
            }

            let title_max_len = (area.width as usize).saturating_sub(25);
            let title = if post.title.len() > title_max_len {
                format!("{}…", &post.title[..title_max_len.saturating_sub(1)])
            } else {
                post.title.clone()
            };

            let date = post
                .pub_date
                .map(|d| d.format("%m/%d").to_string())
                .unwrap_or_default();

            let feed = post
                .feed_title
                .as_ref()
                .map(|t| {
                    if t.len() > 10 {
                        format!("{}…", &t[..9])
                    } else {
                        t.clone()
                    }
                })
                .unwrap_or_default();

            let title_style = if is_selected {
                Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)
            } else if post.is_read {
                Style::default().fg(theme.subtext())
            } else {
                Style::default().fg(theme.text())
            };

            let cursor = if is_selected { "▶" } else { " " };

            ListItem::new(Line::from(vec![
                Span::styled(cursor, Style::default().fg(theme.accent_primary())),
                Span::styled(format!(" {} ", read_indicator), read_style),
                Span::styled(title, title_style),
                Span::styled(badges, Style::default().fg(theme.warning())),
                Span::styled(format!("  {} ", date), Style::default().fg(theme.overlay())),
                Span::styled(format!("[{}]", feed), Style::default().fg(theme.subtext())),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title)
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(theme.surface()));

    let mut state = ListState::default();
    if is_focused {
        state.select(Some(app.selected_index));
    }
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_article_fullscreen(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let Some(post) = app.posts.get(app.selected_index) else {
        return;
    };

    // Add horizontal padding for better readability
    let padding = if area.width > 120 { 15 } else if area.width > 80 { 8 } else { 2 };
    
    let padded_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(padding),
            Constraint::Min(0),
            Constraint::Length(padding),
        ])
        .split(area)[1];

    // Calculate content width for html2text
    let content_width = padded_area.width.saturating_sub(4) as usize;
    
    let content = post.content.as_deref().unwrap_or("No content available.");
    let text_content = html2text::from_read(content.as_bytes(), content_width.max(40))
        .unwrap_or_else(|_| content.to_string());

    let styled_lines = parse_content_to_styled_lines(&text_content, theme);

    let mut title_badges = Vec::new();
    if post.is_bookmarked {
        title_badges.push("★");
    }
    if post.is_read_later {
        title_badges.push("󰃰");
    }
    if post.is_archived {
        title_badges.push("󰆧");
    }

    let title_text = if title_badges.is_empty() {
        post.title.clone()
    } else {
        format!("{} {}", post.title, title_badges.join(" "))
    };

    // Add metadata line
    let feed_name = post.feed_title.as_deref().unwrap_or("Unknown");
    let date = post
        .pub_date
        .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default();

    let mut all_lines = vec![
        Line::from(Span::styled(
            format!("󰉋 {}  │  󰃰 {}", feed_name, date),
            Style::default().fg(theme.subtext()),
        )),
        Line::from(""),
    ];
    all_lines.extend(styled_lines);

    let paragraph = Paragraph::new(all_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(format!(" {} ", title_text))
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD))
                .padding(ratatui::widgets::Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: true })
        .scroll((app.scroll_offset, 0));

    f.render_widget(paragraph, padded_area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let keys = if let Some(msg) = &app.message {
        format!(" {} ", msg)
    } else {
        match (&app.input_mode, &app.focus) {
            (InputMode::Normal, FocusPane::Sidebar) => {
                " h/l:Focus │ j/k:Nav │ Enter:Select │ a:Add Feed │ n:New Cat │ e:Edit Feeds │ d:Del │ ? ".to_string()
            }
            (InputMode::Normal, FocusPane::Posts) => {
                " h/l:Focus │ j/k:Nav │ Enter:Read │ b:Star │ l:Later │ m:Read │ d:Del │ r:Refresh ".to_string()
            }
            (InputMode::Normal, FocusPane::Article) => {
                " Esc:Back │ j/k:Scroll │ b:Star │ l:Later │ a:Archive │ o:Browser │ y:Copy URL ".to_string()
            }
            (InputMode::AddingFeed, _) | (InputMode::AddingCategory, _) => {
                " Type text │ Enter:Confirm │ Esc:Cancel ".to_string()
            }
            (InputMode::SelectingCategory, _) => {
                " j/k:Navigate │ Enter:Select │ Esc:Cancel ".to_string()
            }
            (InputMode::EditingCategoryFeeds(_), _) => {
                " j/k:Navigate │ a:Add Feed │ d:Delete Feed │ Esc:Back ".to_string()
            }
            _ => String::new(),
        }
    };

    let style = if app.message.is_some() {
        Style::default().fg(theme.base()).bg(theme.warning())
    } else {
        Style::default().fg(theme.text()).bg(theme.mantle())
    };

    let status = Paragraph::new(keys).style(style);
    f.render_widget(status, area);
}

fn draw_welcome(f: &mut Frame, _app: &App, area: Rect, theme: &dyn Theme) {
    let welcome_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "󰑫 Welcome to News Reader",
            Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "No feeds configured yet. Get started:",
            Style::default().fg(theme.text()),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [a] ", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled("Add a feed URL", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [i] ", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled("Import from OPML file", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [q] ", Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
            Span::styled("Quit", Style::default().fg(theme.text())),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "OPML files are searched in ~/Downloads/",
            Style::default().fg(theme.subtext()).add_modifier(Modifier::ITALIC),
        )),
    ];

    let paragraph = Paragraph::new(welcome_text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" 󰋜 Setup ")
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
    );

    let popup_area = centered_rect(50, 50, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn draw_input_modal(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme, title: &str) {
    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let input_text = &app.text_input.value;
    let cursor_pos = app.text_input.cursor_position;

    let display_text = format!(
        "{}█{}",
        &input_text[..cursor_pos],
        &input_text[cursor_pos..]
    );

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(&display_text, Style::default().fg(theme.text()))),
        Line::from(""),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(format!(" {} ", title))
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(paragraph, popup_area);
}

fn draw_category_selector(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let popup_area = centered_rect(40, 50, area);
    f.render_widget(Clear, popup_area);

    let items: Vec<ListItem> = app
        .sidebar
        .categories
        .iter()
        .enumerate()
        .map(|(i, cat)| {
            let is_selected = i == app.sidebar.category_index;
            let style = if is_selected {
                Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            };
            let prefix = if is_selected { "▶ " } else { "  " };
            ListItem::new(Line::from(Span::styled(format!("{}{}", prefix, cat), style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" Select Category ")
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
    );

    let mut state = ListState::default();
    state.select(Some(app.sidebar.category_index));
    f.render_stateful_widget(list, popup_area, &mut state);
}

fn draw_category_feeds_editor(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme, category: &str) {
    let popup_area = centered_rect(70, 70, area);
    f.render_widget(Clear, popup_area);

    if app.category_feeds.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No feeds in this category",
                Style::default().fg(theme.subtext()),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'a' to add a feed, or Esc to go back",
                Style::default().fg(theme.overlay()),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(format!(" Feeds in '{}' ", category))
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
        );
        f.render_widget(empty_msg, popup_area);
        return;
    }

    let items: Vec<ListItem> = app
        .category_feeds
        .iter()
        .enumerate()
        .map(|(i, feed)| {
            let is_selected = i == app.category_feed_index;
            let title = feed.title.as_deref().unwrap_or("(No title)");
            let url = if feed.url.len() > 50 {
                format!("{}…", &feed.url[..49])
            } else {
                feed.url.clone()
            };

            let style = if is_selected {
                Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text())
            };

            let cursor = if is_selected { "▶ " } else { "  " };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(cursor, Style::default().fg(theme.accent_primary())),
                    Span::styled(title, style),
                ]),
                Line::from(Span::styled(
                    format!("    {}", url),
                    Style::default().fg(theme.subtext()),
                )),
            ])
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(format!(" Feeds in '{}' ({}) ", category, app.category_feeds.len()))
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
    );

    let mut state = ListState::default();
    state.select(Some(app.category_feed_index));
    f.render_stateful_widget(list, popup_area, &mut state);
}

fn draw_confirm_modal(f: &mut Frame, area: Rect, theme: &dyn Theme, message: &str) {
    let popup_area = centered_rect(40, 20, area);
    f.render_widget(Clear, popup_area);

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(theme.warning()))),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y] ", Style::default().fg(theme.accent_primary())),
            Span::styled("Yes", Style::default().fg(theme.text())),
            Span::raw("    "),
            Span::styled("[n] ", Style::default().fg(theme.accent_primary())),
            Span::styled("No", Style::default().fg(theme.text())),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning()))
            .title(" Confirm ")
            .title_style(Style::default().fg(theme.warning()).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(paragraph, popup_area);
}

fn draw_help_overlay(f: &mut Frame, area: Rect, theme: &dyn Theme) {
    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(Span::styled("Navigation", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))),
        Line::from("  h/l         Focus left/right pane"),
        Line::from("  j/k         Navigate up/down"),
        Line::from("  Enter       Select/Open item"),
        Line::from("  Esc         Go back / Cancel"),
        Line::from(""),
        Line::from(Span::styled("Sidebar", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))),
        Line::from("  a / +       Add new feed (with category selection)"),
        Line::from("  n           Add new category"),
        Line::from("  e           Edit category feeds (view/delete feeds)"),
        Line::from("  d           Delete selected category"),
        Line::from(""),
        Line::from(Span::styled("Posts List", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))),
        Line::from("  b           Toggle bookmark/star"),
        Line::from("  l           Toggle read later"),
        Line::from("  a           Toggle archive"),
        Line::from("  m           Toggle read/unread"),
        Line::from("  d           Delete post"),
        Line::from("  r           Refresh feeds"),
        Line::from("  u           Toggle show/hide read posts"),
        Line::from(""),
        Line::from(Span::styled("Article View", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))),
        Line::from("  j/k         Scroll content"),
        Line::from("  PgUp/PgDn   Scroll faster"),
        Line::from("  o           Open in browser"),
        Line::from("  y           Copy URL to clipboard"),
        Line::from(""),
        Line::from(Span::styled("General", Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))),
        Line::from("  ?           Toggle this help"),
        Line::from("  q           Quit application"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(theme.subtext()))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(" Help ")
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD)),
        );

    f.render_widget(paragraph, popup_area);
}

fn parse_content_to_styled_lines<'a>(content: &'a str, theme: &'a dyn Theme) -> Vec<Line<'a>> {
    content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                Line::from(Span::styled(
                    trimmed.to_string(),
                    Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD),
                ))
            } else if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
                Line::from(vec![
                    Span::styled("  • ", Style::default().fg(theme.accent_primary())),
                    Span::styled(trimmed[2..].to_string(), Style::default().fg(theme.text())),
                ])
            } else if trimmed.starts_with("> ") {
                Line::from(Span::styled(
                    format!("│ {}", &trimmed[2..]),
                    Style::default().fg(theme.subtext()).add_modifier(Modifier::ITALIC),
                ))
            } else {
                Line::from(Span::styled(line.to_string(), Style::default().fg(theme.text())))
            }
        })
        .collect()
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
