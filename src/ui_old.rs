use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, CurrentScreen};
use crate::theme::{Theme, ThemeVariant};
use html2text;

pub fn ui(f: &mut Frame, app: &mut App) {
    // For now, use Claude Code theme by default
    // This will be configurable in Phase 6
    let theme_variant = ThemeVariant::ClaudeCode;
    let theme = theme_variant.get_theme();

    // Set background color for the whole area
    let size = f.area();
    let block = Block::default().style(Style::default().bg(theme.base()));
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    if app.is_loading {
        draw_loading(f, chunks[0], &*theme);
    } else {
        match app.current_screen {
            CurrentScreen::Home => draw_home(f, app, chunks[0], &*theme),
            CurrentScreen::Article => draw_article(f, app, chunks[0], &*theme),
        }
    }

    draw_status_bar(f, app, chunks[1], &*theme);
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
        "               ",
        " Loading feeds...",
    ];

    let text_art: Vec<Line> = art
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(theme.accent_primary()).add_modifier(Modifier::BOLD))))
        .collect();

    let paragraph = Paragraph::new(text_art)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    let area = centered_rect(50, 50, area); // Adjust size as needed
    f.render_widget(paragraph, area);
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

fn draw_home(f: &mut Frame, app: &mut App, area: Rect, theme: &dyn Theme) {
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
                Style::default().fg(theme.accent_primary())
            );

            let bookmark = if post.is_bookmarked {
                Span::styled(" ★", Style::default().fg(theme.warning()))
            } else {
                Span::raw("")
            };

            ListItem::new(Line::from(vec![title, feed, bookmark]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary()))
            .title(" News Feed ")
            .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD))
            .bg(theme.base())
        )
        .highlight_style(Style::default().bg(theme.highlight()).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut ratatui::widgets::ListState::default().with_selected(Some(app.selected_index)));
}

fn draw_article(f: &mut Frame, app: &mut App, area: Rect, theme: &dyn Theme) {
    if let Some(post) = app.posts.get(app.selected_index) {
        let content_raw = post.content.clone().unwrap_or_default();
        let display_content = if content_raw.trim().is_empty() {
             format!("No content available for this post.\n\nLink: {}", post.url)
        } else {
             content_raw
        };

        // Convert HTML to Text
        let text_content = html2text::from_read(display_content.as_bytes(), area.width as usize)
             .unwrap_or_else(|_| format!("Error parsing content.\n\nLink: {}", post.url));

        let title_text = if post.is_bookmarked {
            format!("{} ★", post.title)
        } else {
            post.title.clone()
        };

        let paragraph = Paragraph::new(text_content)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent_primary()))
                .title(title_text.as_str())
                .title_style(Style::default().fg(theme.accent_secondary()).add_modifier(Modifier::BOLD))
                .bg(theme.base())
            )
            .style(Style::default().fg(theme.text()))
            .wrap(Wrap { trim: true })
            .scroll((app.scroll_offset, 0));

        f.render_widget(paragraph, area);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect, theme: &dyn Theme) {
    let mode = if let Some(msg) = &app.message {
        format!("MSG: {} (Press any key to dismiss)", msg)
    } else {
        match app.current_screen {
            CurrentScreen::Home => "HOME (Enter: Read, b: Bookmark, u: Toggle Unread, s: Toggle Saved, q: Quit, r: Refresh)",
            CurrentScreen::Article => "ARTICLE (Esc: Back, b: Bookmark, Up/Down: Scroll)",
        }.to_string()
    };

    let style = if app.message.is_some() {
        Style::default().fg(theme.base()).bg(theme.warning()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text()).bg(theme.mantle())
    };

    let status = Paragraph::new(mode).style(style);
    f.render_widget(status, area);
}