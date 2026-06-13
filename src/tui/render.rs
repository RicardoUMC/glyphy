use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
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
    let (chunks, main_chunks) = layout_chunks(area);

    // Title bar at the very top.
    render_title_bar(frame, chunks[0], app);

    crate::tui::widgets::render_image(frame, main_chunks[0], app);
    crate::tui::widgets::render_settings(frame, main_chunks[1], app);

    // Status bar at the bottom.
    crate::tui::widgets::render_status_bar(frame, chunks[2], app);

    // Help overlay drawn on top of everything when visible.
    if app.show_help {
        crate::tui::widgets::render_help_overlay(frame, area);
    }
}

/// Resolve the drawable image area inside the output pane.
pub fn image_inner_area(area: Rect) -> Rect {
    let (_, main_chunks) = layout_chunks(area);
    main_chunks[0].inner(Margin {
        vertical: 1,
        horizontal: 1,
    })
}

fn layout_chunks(area: Rect) -> (std::rc::Rc<[Rect]>, std::rc::Rc<[Rect]>) {
    // Top-level vertical layout: title, content, status bar.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Main content: horizontal split — image output (left) + settings (right).
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70), // Image output
            Constraint::Percentage(30), // Settings panel
        ])
        .split(chunks[1]);

    (chunks, main_chunks)
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
