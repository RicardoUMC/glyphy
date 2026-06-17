# Tasks: TUI UX Polish

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | 450-650 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 theme/layout -> PR 2 settings -> PR 3 focus/help/tests |
| Delivery strategy | ask-on-risk |
| Chain strategy | feature-branch-chain |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: feature-branch-chain
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Notes |
|------|------|-----------|-------|
| 1 | Theme palette and responsive layout foundation | PR 1 | Review visual foundation first; include palette/layout tests. |
| 2 | Interactive settings controls | PR 2 | Depends on PR 1; include reprocess/redraw tests. |
| 3 | Explicit focus, contextual help/status, smoke checks | PR 3 | Depends on PR 2; preserve picker behavior. |

## Phase 1: Theme And Responsive Layout

- [x] 1.1 Create `src/tui/theme.rs` with `ThemeMode`, `Theme`, semantic styles, dark/light palettes, and pure palette tests.
- [x] 1.2 Export `theme` from `src/tui/mod.rs`; add `theme_mode: ThemeMode` to `src/tui/app.rs` without changing `Config`.
- [x] 1.3 Update `src/tui/render.rs` to derive compact/normal/wide and short layouts from terminal size thresholds.
- [x] 1.4 Replace inline TUI colors in `src/tui/render.rs` and `src/tui/widgets.rs` with semantic theme styles.

## Phase 2: Interactive Settings Controls

- [ ] 2.1 Add `selected_setting_index` and local setting-control metadata for width, height, ramp, invert, and theme.
- [ ] 2.2 Render selected controls in `src/tui/widgets.rs` as interactive rows/chips with current values and disabled states where needed.
- [ ] 2.3 Wire settings actions in `src/tui/app.rs` so width/height/ramp/invert reprocess preview and theme redraws chrome only.
- [ ] 2.4 Extend `src/tui/keys.rs` actions only as needed for setting selection, adjustment, toggle, and theme cycling.

## Phase 3: Focus And Navigation Model

- [ ] 3.1 Replace `focus: char` in `src/tui/app.rs` with `Focus::{Files, Settings, Output}` and remove invalid focus states.
- [ ] 3.2 Make `h/j/k/l` and arrow handling contextual: settings navigate/adjust, output resizes, picker navigation stays unchanged.
- [ ] 3.3 Update focused panel styling in `src/tui/widgets.rs` so file/settings/output focus is visually clear.
- [ ] 3.4 Verify returning from picker sets output focus and reopening picker preserves picker CWD, sorting, and selection semantics.

## Phase 4: Help And Status Polish

- [ ] 4.1 Update `src/tui/widgets.rs` status/help text to reflect only actions valid for the current focus.
- [ ] 4.2 Create a shared keybinding/command registry in `src/tui/keys.rs` only if help text duplicates behavior enough to drift.
- [ ] 4.3 Report setting changes with fresh status values after the action completes.

## Phase 5: Verification

- [ ] 5.1 Add unit tests for layout thresholds, theme readability states, focus transitions, setting bounds, and contextual key conflicts.
- [ ] 5.2 Keep existing `src/tui/app.rs` and `src/tui/keys.rs` picker/ramp/invert tests passing; add regressions for picker preservation.
- [ ] 5.3 Run `cargo test` and `cargo check`.
- [ ] 5.4 Manual smoke: dark/light, compact `<80`, normal `80..120`, wide `>=120`, short `<20`, help overlay, quit, picker select/return.
