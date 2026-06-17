use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyEventKind};

use crate::tui::app::App;
use crate::tui::keys::resolve_key;

/// Poll for terminal events and dispatch them.
pub fn handle_events(app: &mut App) -> Result<()> {
    let event = if app.dirty {
        if !event::poll(Duration::from_millis(100))? {
            return Ok(());
        }
        event::read()?
    } else {
        event::read()?
    };

    match event {
        Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
            if let Some(action) = resolve_key(key) {
                app.handle_action(action);
            }
        }
        Event::Resize(_, _) if app.auto_size => {
            let _ = app.process();
        }
        _ => {}
    }
    Ok(())
}
