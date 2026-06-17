# TUI UX Specification

## Purpose

Define polished TUI visual, responsive, settings, focus, help, and status behavior while preserving CLI and picker behavior.

## Requirements

### Requirement: Theme Modes

The TUI MUST support dark and light theme modes and MUST provide a foundation for personal presets. Themes MUST be attractive, sober, and readable across common terminals.

#### Scenario: Select theme mode

- GIVEN the user is in the TUI settings context
- WHEN the user changes the theme selector
- THEN panels, status, help, and controls use the selected theme
- AND image processing output remains unchanged

#### Scenario: Light mode remains readable

- GIVEN light mode is active
- WHEN the TUI renders normal, selected, disabled, and status text
- THEN each state MUST remain visually distinguishable on a light terminal background

### Requirement: Responsive Layout

The TUI MUST adapt to terminal size changes in real time. Normal sizes SHOULD show file, settings, and output areas without crowding. Narrow or short sizes MUST preserve essential actions and MUST NOT render cramped or overlapping panels.

#### Scenario: Resize to normal terminal

- GIVEN the TUI is open
- WHEN the terminal has enough width and height for all primary areas
- THEN file, settings, output, help, and status are visible in a balanced layout

#### Scenario: Resize to narrow terminal

- GIVEN the TUI is open
- WHEN terminal width becomes narrow
- THEN the layout collapses or simplifies non-essential detail
- AND navigation, selection, settings, help, and quit remain discoverable

### Requirement: Interactive Settings Controls

The TUI MUST expose interactive controls: width stepper, height stepper when applicable, ramp selector, invert toggle, and theme selector. Changes MUST update preview without restart.

#### Scenario: Adjust size control

- GIVEN an image is loaded and the width control is focused
- WHEN the user increments or decrements the control
- THEN preview is reprocessed with the new width
- AND the current control remains selected

#### Scenario: Toggle visual options

- GIVEN the ramp, invert, or theme control is focused
- WHEN the user changes the selected control
- THEN the setting value updates
- AND the preview or TUI chrome reflects the change immediately

### Requirement: Focus And Navigation

The TUI MUST provide predictable focus between file, settings, and output contexts. Navigation keys MUST be contextual. Conflicting keys such as `h`, `j`, `k`, and `l` MUST NOT perform unrelated actions in one context.

#### Scenario: Move focus predictably

- GIVEN the TUI has file, settings, and output contexts available
- WHEN the user moves focus between contexts
- THEN the focused context is visually clear
- AND available controls apply only to that context

#### Scenario: Resolve hjkl conflict

- GIVEN a settings control is focused
- WHEN the user presses `h`, `j`, `k`, or `l`
- THEN the key MUST perform the documented contextual settings action
- AND MUST NOT also trigger output resizing or file navigation

#### Scenario: Preserve file picker behavior

- GIVEN the file picker is focused
- WHEN the user navigates directories, selects an image, or returns to the picker
- THEN behavior MUST match the file-picker specification
- AND focus changes MUST NOT alter picker CWD, sorting, or selection semantics

### Requirement: Help And Status UX

The TUI MUST display help and status text for current contextual actions. Help/status content MUST NOT drift from actual key behavior.

#### Scenario: Contextual help changes with focus

- GIVEN focus is on file, settings, or output
- WHEN the focused context changes
- THEN help text updates to show actions valid for the new context
- AND invalid actions for that context are hidden or clearly unavailable

#### Scenario: Status reports setting changes

- GIVEN the user changes a setting
- WHEN the TUI finishes applying the change
- THEN status text SHOULD describe the current result or selected value
- AND MUST NOT report stale values.
