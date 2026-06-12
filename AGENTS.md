# Glyphy

Terminal-first image renderer using Unicode/ASCII glyphs. Rust, MVP phase.

## Build & Run

```bash
cargo check          # fast type-check
cargo build          # debug build
cargo run -- -i img.png -w 80   # render at 80 chars wide
```

No tests exist yet. No CI. No formatter config beyond rustfmt defaults.

## Architecture

Layered, trait-based. Layers must not import from each other except through traits.

```
input/       → ImageSource trait (load image from path)
processing/  → Processor trait + GlyphBuffer (image → glyph grid)
rendering/   → Renderer trait (glyph grid → output)
app/         → Pipeline orchestrator (wires input→process→render)
config.rs    → Config struct + brightness_to_char mapping
```

**Concrete implementations live in `main.rs`**, not in their modules. This is intentional for MVP — keeps the library surface minimal. Move them to their modules when adding alternative implementations.

`GlyphBuffer` is the contract between processing and rendering. Processors produce it, renderers consume it. It holds a `Vec<Vec<GlyphCell>>` in row-major order.

## Constraints

- No video support in first iteration
- No webcam support in first iteration
- No advanced export in first iteration
- Architectural clarity over premature optimization
- Avoid unnecessary dependencies
- Processing must not depend on UI
- Renderer must be reusable for both stdout and future TUI

## Key Gotchas

- **Terminal aspect ratio**: Characters are ~2:1 (height:width). The processor compensates by multiplying target width by 0.5 when calculating height. If you change the resize logic, preserve this factor or output will be vertically stretched.
- **Brightness formula**: Uses ITU-R BT.601 (0.299R + 0.587G + 0.114B). Changing this affects which characters map to which brightness levels.
- **Ramp ordering**: `Config.ramp` goes dark→bright (space first, dense char last). `brightness_to_char()` indexes into it. Inverted mode flips the brightness before indexing.
- **Buffer output**: `GlyphBuffer::to_string_output()` builds a single `String` before printing. Do NOT print per-character — it's orders of magnitude slower.

## Dependencies

Minimal by design:
- `image` 0.25 — load/resize PNG/JPG/WEBP
- `clap` 4 (derive) — CLI parsing
- `crossterm` 0.28 — terminal clear/cursor (only used in TerminalRenderer)
- `anyhow` 1 — error handling

Do not add `ratatui`, `tokio`, `log`, or `tracing` until the roadmap requires it.

## Roadmap

| Phase | Goal | Status |
|-------|------|--------|
| 1 | Image → Glyphs → Terminal | ✓ done |
| 2 | Interactive config (width, height, char ramp, invert) | next |
| 3 | TUI with ratatui | planned |
| 4 | Real-time video | planned |
| 5 | ANSI colors | planned |
| 6 | Unicode advanced + Braille rendering | planned |
