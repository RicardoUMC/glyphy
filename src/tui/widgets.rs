use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

/// Render the image output area showing the GlyphBuffer content.
pub fn render_image(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("Output");
    if let Some(err) = &app.last_error {
        let text = Text::from(Line::from(Span::styled(
            err,
            Style::default().fg(Color::Red),
        )));
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    } else if let Some(buffer) = &app.buffer {
        let text = buffer.to_string_output();
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No image loaded.").block(block);
        frame.render_widget(paragraph, area);
    }
}

/// Render the settings panel showing current config and keybinding hints.
pub fn render_settings(frame: &mut Frame, area: Rect, app: &App) {
    let config = &app.config;
    let ramp_str: String = config.ramp.iter().collect();

    let lines = vec![
        Line::from(format!("Width:  {}", config.width.unwrap_or(80))),
        Line::from(format!("Height: {}", config.height.unwrap_or(40))),
        Line::from(format!("Ramp:   {}", ramp_str)),
        Line::from(format!("Invert: {}", if config.invert { "on" } else { "off" })),
        Line::from(""),
        Line::from("── Keys ──"),
        Line::from("  h/l  adjust width"),
        Line::from("  j/k  adjust height"),
        Line::from("  r    cycle ramp"),
        Line::from("  i    toggle invert"),
        Line::from("  ?    help  ·  q  quit"),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render the bottom status bar with keybinding hints.
pub fn render_status_bar(frame: &mut Frame, area: Rect, _app: &App) {
    let text = Line::from(Span::styled(
        " q quit  |  ? help  |  ← → ↑ ↓ / hjkl  |  r ramp  |  i invert",
        Style::default().fg(Color::Black).bg(Color::White),
    ));
    frame.render_widget(Paragraph::new(text), area);
}

/// Render a centered help dialog overlay.
pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(55, 55, area);

    let help_text = Text::from(vec![
        Line::from(" Help — Glyphy TUI"),
        Line::from(""),
        Line::from(" q / Ctrl+C / Esc   Quit"),
        Line::from(" h / ←              Decrease width"),
        Line::from(" l / →              Increase width"),
        Line::from(" j / ↓              Decrease height"),
        Line::from(" k / ↑              Increase height"),
        Line::from(" r                  Cycle ramp preset"),
        Line::from(" i                  Toggle invert"),
        Line::from(" ?                  Close this help"),
    ]);

    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(Clear, popup);
    frame.render_widget(help, popup);
}

/// Compute a centered rectangle within `r` given percentage dimensions.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Percentage(percent_y),
        Constraint::Fill(1),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .split(vertical[1])[1]
}
