use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;

/// Compose all widgets into the terminal layout and render a single frame.
///
/// Layout:
/// ┌─────────────────────────────────────────────┐
/// │  Title bar (Glyphy)          [?] help       │
/// ├──────────────────────┬──────────────────────┤
/// │                      │  Settings panel      │
/// │  Image output        │  (config + keys)     │
/// │                      │                      │
/// ├──────────────────────┴──────────────────────┤
/// │  Status bar (keybinding hints)              │
/// └─────────────────────────────────────────────┘
///
/// When `show_help` is true, a centered overlay dialog is drawn on top.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Top-level vertical layout: title, content, status bar.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Title bar at the very top.
    render_title_bar(frame, chunks[0], app);

    // Main content: horizontal split — image output (left) + settings (right).
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Image output
            Constraint::Percentage(30), // Settings panel
        ])
        .split(chunks[1]);

    crate::tui::widgets::render_image(frame, main_chunks[0], app);
    crate::tui::widgets::render_settings(frame, main_chunks[1], app);

    // Status bar at the bottom.
    crate::tui::widgets::render_status_bar(frame, chunks[2], app);

    // Help overlay drawn on top of everything when visible.
    if app.show_help {
        crate::tui::widgets::render_help_overlay(frame, area);
    }
}

/// Render a single-line title bar with the app name and help hint.
fn render_title_bar(frame: &mut Frame, area: Rect, app: &App) {
    let title = if app.last_error.is_some() {
        " Glyphy (error)"
    } else {
        " Glyphy"
    };
    let hint = "[?] help";

    let text = Line::from(vec![
        Span::styled(title, Style::default().bold().fg(Color::White)),
        Span::styled("  ", Style::default().fg(Color::White)),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(
        Paragraph::new(text).style(Style::default().bg(Color::Black)),
        area,
    );
}
