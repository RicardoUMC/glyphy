# Exploration: TUI UX Polish

## Current State

Functional but still MVP-style:

- Fixed 70/30 layout: title/status bars, file picker or output left, settings right
- Focus exists as `char` (`'f'/'s'/'o'`) but is mostly visual; no contextual navigation except picker mode
- Settings are passive text + key hints. No selected setting row, no interactive controls, no theme abstraction
- Colors are hardcoded inline in `render.rs` and `widgets.rs`

## Best Low-Churn Direction

Introduce a small `Theme` model and richer settings rendering first, then add real `SettingControl` focus/selection behavior in a second change. Use OpenCode as an additional reference for sober aesthetics, terminal-size responsiveness, and real-time adaptive layout behavior.

## New Concepts Needed

- `Theme` / `ThemeMode` (light/dark/personal)
- `SettingControl` with selected row and visible controls
- `Focus` enum instead of `char`
- `selected_setting_index`
- Command/help registry so keybindings and help/status don't drift
- Responsive layout rules that adapt panel proportions and content density to current terminal size

## Staged Plan

### Stage 1: Theme/Palette
- Add `Theme` struct with palette colors
- Replace hardcoded colors in render.rs and widgets.rs
- Light/dark mode toggle

### Stage 2: Interactive Settings
- Width: `[-] 120 [+]` controls
- Height: `[-] 40 [+]` controls
- Ramp: selector/preset preview
- Invert: toggle chip
- Theme: light/dark/personal preset selector

### Stage 3: Navigation Model
- Explicit `Focus` enum instead of `char`
- Optional `selected_setting_index`
- Contextual key handling based on focused panel

### Stage 4: Polish
- Help/status with shared keybinding/command registry
- Better visual hierarchy
- Responsive behavior inspired by OpenCode: preserve clarity across terminal sizes, avoid cramped panels, and adapt detail density in real time

## Risks

- Hardcoded colors scattered in render.rs and widgets.rs
- Light mode needs careful contrast; terminals vary wildly
- Responsive behavior needs careful thresholds so panels do not jump or hide important controls unexpectedly
- `h/j/k/l` globally resize image; if settings become navigable, key semantics may conflict
- More interactive settings increases App state complexity unless Focus and SettingControl are introduced cleanly

## Review Workload Forecast

| Change | Estimated Lines | Risk | Chain PRs? |
|--------|----------------|------|------------|
| Theme-only | 120–220 | Low | No |
| Interactive settings | 250–450 | Medium | Maybe |
| Full polish + registry | 500+ | High | Yes |
