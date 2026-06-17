use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::app::App;
use crate::tui::theme::Theme;

const COMPACT_WIDTH: u16 = 80;
const WIDE_WIDTH: u16 = 120;
const SHORT_HEIGHT: u16 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutWidth {
    Compact,
    Normal,
    Wide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutMode {
    pub width: LayoutWidth,
    pub short: bool,
}

impl LayoutMode {
    pub fn from_area(area: Rect) -> Self {
        let width = if area.width < COMPACT_WIDTH {
            LayoutWidth::Compact
        } else if area.width < WIDE_WIDTH {
            LayoutWidth::Normal
        } else {
            LayoutWidth::Wide
        };

        Self {
            width,
            short: area.height < SHORT_HEIGHT,
        }
    }
}

/// Compose all widgets into the terminal layout and render a single frame.
///
/// Layout:
/// ┌─────────────────────────────────────────────┐
/// │  Title bar (Glyphy)          [?] help       │
/// ├──────────────────────┬──────────────────────┤
/// │                      │  Settings panel      │
/// │  Image output        │  (config + keys)     │
/// │  or File picker      │                      │
/// │                      │                      │
/// ├──────────────────────┴──────────────────────┤
/// │  Status bar (keybinding hints)              │
/// └─────────────────────────────────────────────┘
///
/// When `show_help` is true, a centered overlay dialog is drawn on top.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let (chunks, main_chunks) = layout_chunks(area);
    let theme = Theme::from_mode(app.theme_mode);

    // Title bar at the very top.
    render_title_bar(frame, chunks[0], app, &theme);

    // Left panel: file picker or image output
    if app.picker_mode {
        crate::tui::widgets::render_picker(frame, main_chunks[0], app, &theme);
    } else {
        crate::tui::widgets::render_image(frame, main_chunks[0], app, &theme);
    }

    crate::tui::widgets::render_settings(frame, main_chunks[1], app, &theme);

    // Status bar at the bottom.
    crate::tui::widgets::render_status_bar(frame, chunks[2], app, &theme);

    // Help overlay drawn on top of everything when visible.
    if app.show_help {
        crate::tui::widgets::render_help_overlay(frame, area, &theme);
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
    let mode = LayoutMode::from_area(area);

    // Top-level vertical layout: title, content, status bar.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let main_chunks = match mode.width {
        LayoutWidth::Compact => Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(settings_height(mode)),
            ])
            .split(chunks[1]),
        LayoutWidth::Normal => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(chunks[1]),
        LayoutWidth::Wide => Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(chunks[1]),
    };

    (chunks, main_chunks)
}

fn settings_height(mode: LayoutMode) -> u16 {
    if mode.short {
        5
    } else {
        9
    }
}

/// Render a single-line title bar with the app name and help hint.
fn render_title_bar(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let title = if app.last_error.is_some() {
        " Glyphy (error)"
    } else if app.picker_mode {
        " Glyphy — Select Image"
    } else {
        " Glyphy"
    };
    let hint = "[?] help";

    let mut spans = vec![
        Span::styled(title, theme.accent),
        Span::styled("  ", theme.text),
    ];

    // Show CWD when in picker mode
    if app.picker_mode {
        let cwd_display = app.picker_cwd.display().to_string();
        // Truncate if too long
        let max_len = area.width.saturating_sub(30) as usize;
        let cwd_str = if cwd_display.len() > max_len && max_len > 3 {
            format!("...{}", &cwd_display[cwd_display.len() - max_len + 3..])
        } else {
            cwd_display
        };
        spans.push(Span::styled(cwd_str, theme.muted));
    }

    spans.push(Span::styled("  ", theme.text));
    spans.push(Span::styled(hint, theme.muted));

    frame.render_widget(Paragraph::new(Line::from(spans)).style(theme.chrome), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_mode_is_compact_below_80_columns() {
        let mode = LayoutMode::from_area(Rect::new(0, 0, 79, 24));

        assert_eq!(mode.width, LayoutWidth::Compact);
        assert!(!mode.short);
    }

    #[test]
    fn layout_mode_is_normal_from_80_to_119_columns() {
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 80, 24)).width,
            LayoutWidth::Normal
        );
        assert_eq!(
            LayoutMode::from_area(Rect::new(0, 0, 119, 24)).width,
            LayoutWidth::Normal
        );
    }

    #[test]
    fn layout_mode_is_wide_at_120_columns() {
        let mode = LayoutMode::from_area(Rect::new(0, 0, 120, 24));

        assert_eq!(mode.width, LayoutWidth::Wide);
    }

    #[test]
    fn layout_mode_marks_short_below_20_rows() {
        let mode = LayoutMode::from_area(Rect::new(0, 0, 100, 19));

        assert!(mode.short);
    }

    #[test]
    fn compact_layout_stacks_output_above_settings() {
        let (_chunks, main_chunks) = layout_chunks(Rect::new(0, 0, 79, 24));

        assert_eq!(main_chunks.len(), 2);
        assert_eq!(main_chunks[0].x, 0);
        assert_eq!(main_chunks[1].x, 0);
        assert!(main_chunks[1].y > main_chunks[0].y);
        assert_eq!(main_chunks[1].height, 9);
    }

    #[test]
    fn short_compact_layout_uses_smaller_settings_area() {
        let (_chunks, main_chunks) = layout_chunks(Rect::new(0, 0, 79, 19));

        assert_eq!(main_chunks[1].height, 5);
        assert!(main_chunks[0].height >= 5);
    }

    #[test]
    fn normal_layout_splits_horizontally_with_larger_output_area() {
        let (_chunks, main_chunks) = layout_chunks(Rect::new(0, 0, 100, 24));

        assert_eq!(main_chunks.len(), 2);
        assert_eq!(main_chunks[0].y, main_chunks[1].y);
        assert!(main_chunks[0].width > main_chunks[1].width);
        assert_eq!(main_chunks[0].width, 65);
        assert_eq!(main_chunks[1].width, 35);
    }

    #[test]
    fn wide_layout_gives_more_space_to_output_area() {
        let (_chunks, main_chunks) = layout_chunks(Rect::new(0, 0, 120, 24));

        assert_eq!(main_chunks[0].y, main_chunks[1].y);
        assert_eq!(main_chunks[0].width, 90);
        assert_eq!(main_chunks[1].width, 30);
    }
}
