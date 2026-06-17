use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::Theme;

/// Render the image output area showing the GlyphBuffer content.
pub fn render_image(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_style = if app.focus == 'o' {
        theme.focused_border
    } else {
        theme.text
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Output")
        .border_style(border_style);

    if let Some(err) = &app.last_error {
        let text = Text::from(Line::from(Span::styled(err, theme.error)));
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

/// Render the file picker panel showing CWD directories and image files.
pub fn render_picker(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_style = if app.focus == 'f' {
        theme.file_border
    } else {
        theme.text
    };

    // Show index indicator in title
    let title = if app.picker_entries.is_empty() {
        "Files".to_string()
    } else {
        format!(
            "Files [{}/{}]",
            app.picker_index + 1,
            app.picker_entries.len()
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style);

    if app.picker_entries.is_empty() {
        let text = Text::from(vec![
            Line::from("No image files found"),
            Line::from("in current directory."),
            Line::from(""),
            Line::from(Span::styled("Place images here or use", theme.muted)),
            Line::from(Span::styled("glyphy -i <file> --tui", theme.muted)),
        ]);
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<Line> = app
        .picker_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let name = entry.name();
            let style = if i == app.picker_index {
                theme.selection
            } else if entry.is_dir() {
                theme.directory
            } else {
                theme.text
            };
            Line::from(Span::styled(format!("  {}", name), style))
        })
        .collect();

    let paragraph = Paragraph::new(Text::from(items))
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render the settings panel showing current config and keybinding hints.
pub fn render_settings(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let border_style = if app.focus == 's' {
        theme.focused_border
    } else {
        theme.text
    };

    let config = &app.config;
    let ramp_str: String = config.ramp.iter().collect();
    let compact = area.width < 50 || area.height < 10;

    let mut lines = vec![
        Line::from(format!("Width:  {}", config.width.unwrap_or(80))),
        Line::from(format!("Height: {}", config.height.unwrap_or(40))),
        Line::from(format!("Ramp:   {}", ramp_str)),
        Line::from(vec![
            Span::raw("Invert: "),
            if config.invert {
                Span::styled("on", theme.settings_on)
            } else {
                Span::styled("off", theme.settings_off)
            },
        ]),
    ];

    if compact {
        lines.push(Line::from(Span::styled(
            "h/l width · j/k height · r ramp · i invert · ? help",
            theme.muted,
        )));
    } else {
        lines.extend([
            Line::from(""),
            Line::from(Span::styled("-- Keys --", theme.accent)),
            Line::from("  h/l  adjust width"),
            Line::from("  j/k  adjust height"),
            Line::from("  r    cycle ramp"),
            Line::from("  i    toggle invert"),
            Line::from("  ?    help  ·  q  quit"),
            Line::from(""),
            Line::from(Span::styled("-- Sections --", theme.accent)),
            Line::from("  f    files panel"),
            Line::from("  s    settings panel"),
            Line::from("  o    output panel"),
            Line::from("  ⌫    back to picker"),
        ]);
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Settings")
                .border_style(border_style),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render the bottom status bar with keybinding hints.
pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mode = if app.picker_mode {
        Span::styled(" PICKER ", theme.picker_badge)
    } else {
        Span::styled(" VIEW ", theme.view_badge)
    };

    let focus = match app.focus {
        'f' => Span::styled(" [f]ile ", theme.picker_badge),
        's' => Span::styled(" [s]ettings ", theme.view_badge),
        _ => Span::styled(" [o]utput ", theme.view_badge),
    };

    let hints = if app.picker_mode {
        Span::styled("  j/k navigate · Enter select · q quit", theme.muted)
    } else {
        Span::styled("  hjkl resize · r ramp · i invert · ⌫ picker", theme.muted)
    };

    let text = Line::from(vec![mode, focus, hints]);
    frame.render_widget(Paragraph::new(text).style(theme.chrome), area)
}

/// Render a centered help dialog overlay.
pub fn render_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup = centered_rect(55, 55, area);

    let help_text = Text::from(vec![
        Line::from(Span::styled(" Help — Glyphy TUI", theme.accent)),
        Line::from(""),
        Line::from(" q / Ctrl+C / Esc   Quit"),
        Line::from(" h / ←              Decrease width"),
        Line::from(" l / →              Increase width"),
        Line::from(" j / ↓              Decrease height"),
        Line::from(" k / ↑              Increase height"),
        Line::from(" r                  Cycle ramp preset"),
        Line::from(" i                  Toggle invert"),
        Line::from(" f                  Focus file panel"),
        Line::from(" s                  Focus settings panel"),
        Line::from(" o                  Focus output panel"),
        Line::from(" ⌫ (Backspace)     Back to file picker"),
        Line::from(" ..                 Go to parent directory"),
        Line::from(" Enter              Select file (picker)"),
        Line::from(" ?                  Close this help"),
    ]);

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(theme.focused_border),
        )
        .style(theme.help_overlay);

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
