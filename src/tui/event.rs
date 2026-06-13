use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event};

use crate::tui::app::App;
use crate::tui::keys::resolve_key;

/// Poll for terminal events with a short timeout and dispatch them.
///
/// This function is non-blocking — it returns after the timeout (100 ms)
/// even if no event is available. Key events are resolved via `resolve_key`
/// and dispatched to `app.handle_action`. All other events are ignored.
pub fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            if let Some(action) = resolve_key(key) {
                app.handle_action(action);
            }
        }
    }
    Ok(())
}
