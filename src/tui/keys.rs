use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Actions that can be triggered by keyboard input.
///
/// Each variant represents a single user intent. Both vim keys
/// and arrow keys map to the same action (e.g., 'h' and ← both
/// produce `WidthDown`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Exit the TUI (q, Ctrl+C, Esc).
    Quit,
    /// Increase image width (l, →).
    WidthUp,
    /// Decrease image width (h, ←).
    WidthDown,
    /// Increase image height (k, ↑).
    HeightUp,
    /// Decrease image height (j, ↓).
    HeightDown,
    /// Cycle to the next character ramp.
    CycleRamp,
    /// Toggle brightness inversion.
    ToggleInvert,
    /// Show or hide the help dialog.
    ToggleHelp,
    /// Confirm selection in picker (Enter).
    NavConfirm,
    /// Focus file panel (f).
    FocusFile,
    /// Focus settings panel (s).
    FocusSettings,
    /// Focus output panel (o).
    FocusOutput,
    /// Back to picker from image view (Backspace).
    BackToPicker,
}

/// Resolve a crossterm [`KeyEvent`] into a [`KeyAction`].
///
/// Returns `None` when the key press does not match any binding.
pub fn resolve_key(event: KeyEvent) -> Option<KeyAction> {
    match (event.code, event.modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE)
        | (KeyCode::Char('c'), KeyModifiers::CONTROL)
        | (KeyCode::Esc, KeyModifiers::NONE) => Some(KeyAction::Quit),

        // Width: h/l or ←/→
        (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, KeyModifiers::NONE) => {
            Some(KeyAction::WidthDown)
        }
        (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, KeyModifiers::NONE) => {
            Some(KeyAction::WidthUp)
        }

        // Height: j/k or ↓/↑
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            Some(KeyAction::HeightDown)
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            Some(KeyAction::HeightUp)
        }

        // Cycle ramp
        (KeyCode::Char('r'), KeyModifiers::NONE) => Some(KeyAction::CycleRamp),

        // Toggle invert
        (KeyCode::Char('i'), KeyModifiers::NONE) => Some(KeyAction::ToggleInvert),

        // Toggle help
        (KeyCode::Char('?'), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
            Some(KeyAction::ToggleHelp)
        }

        // Confirm selection (Enter)
        (KeyCode::Enter, KeyModifiers::NONE) => Some(KeyAction::NavConfirm),

        // Section focus
        (KeyCode::Char('f'), KeyModifiers::NONE) => Some(KeyAction::FocusFile),
        (KeyCode::Char('s'), KeyModifiers::NONE) => Some(KeyAction::FocusSettings),
        (KeyCode::Char('o'), KeyModifiers::NONE) => Some(KeyAction::FocusOutput),

        // Back to picker
        (KeyCode::Backspace, KeyModifiers::NONE) => Some(KeyAction::BackToPicker),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_via_q() {
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(resolve_key(event), Some(KeyAction::Quit));
    }

    #[test]
    fn quit_via_ctrl_c() {
        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(resolve_key(event), Some(KeyAction::Quit));
    }

    #[test]
    fn quit_via_esc() {
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(resolve_key(event), Some(KeyAction::Quit));
    }

    #[test]
    fn width_up_vim_and_arrow() {
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE)),
            Some(KeyAction::WidthUp)
        );
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            Some(KeyAction::WidthUp)
        );
    }

    #[test]
    fn width_down_vim_and_arrow() {
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE)),
            Some(KeyAction::WidthDown)
        );
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            Some(KeyAction::WidthDown)
        );
    }

    #[test]
    fn height_up_vim_and_arrow() {
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
            Some(KeyAction::HeightUp)
        );
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            Some(KeyAction::HeightUp)
        );
    }

    #[test]
    fn height_down_vim_and_arrow() {
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
            Some(KeyAction::HeightDown)
        );
        assert_eq!(
            resolve_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            Some(KeyAction::HeightDown)
        );
    }

    #[test]
    fn cycle_ramp() {
        let event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        assert_eq!(resolve_key(event), Some(KeyAction::CycleRamp));
    }

    #[test]
    fn toggle_invert() {
        let event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);
        assert_eq!(resolve_key(event), Some(KeyAction::ToggleInvert));
    }

    #[test]
    fn toggle_help() {
        let event = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE);
        assert_eq!(resolve_key(event), Some(KeyAction::ToggleHelp));
    }

    #[test]
    fn unknown_key_returns_none() {
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(resolve_key(event), None);
    }
}
