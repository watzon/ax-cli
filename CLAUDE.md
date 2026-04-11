# AGENTS.md

## Project

`ax` — a macOS-only CLI for inspecting the Accessibility (AX) API. Binary is named `ax`, crate name is `ax`.

## macOS-only

The entire project depends on macOS system frameworks (Accessibility, ScreenCaptureKit, CoreFoundation, Cocoa). It will not compile on Linux or Windows. All CI runs on `macos-latest`.

## Developer commands

```bash
# Format check (what CI runs)
cargo fmt --all -- --check

# Auto-fix formatting
cargo fmt --all

# Lint — CI runs with -D warnings (all clippy warnings are errors)
cargo clippy --all-targets --all-features -- -D warnings

# Tests
cargo test

# Release build — binary lands at target/release/ax
cargo build --release
```

CI order: `fmt` → `clippy` → `test` → `build`. Fix in the same order.

## Clippy

`Cargo.toml` sets `[lints.clippy] all = "warn"`. CI escalates to `-D warnings`. Don't introduce new clippy warnings.

## Source layout

```
src/
  main.rs        — entry point, all cmd_* dispatch functions
  cli.rs         — Clap argument structs (Commands, *Args)
  error.rs       — AxError enum; exit codes 1–7 are meaningful
  capture.rs     — ScreenCaptureKit screenshot logic
  ax/            — AX API wrappers
    actions.rs
    app.rs
    attributes.rs
    catalog.rs   — static AX catalog (discover command)
    element.rs
    mod.rs
    mutation.rs
    observer.rs  — CFRunLoop-based notification watching
    parameterized.rs
    tree.rs
  output/        — formatters
    json_fmt.rs
    plain_fmt.rs
    tree_fmt.rs
```

## build.rs

Adds `-Wl,-rpath,/usr/lib/swift` for macOS. Required by the `screencapturekit` crate. Do not remove.

## Runtime permissions (not needed to build/test)

Most commands require **Accessibility** permission for the terminal app. `ax screenshot` additionally requires **Screen Recording** permission. `ax list` and `ax discover` work without either permission.

## Exit codes

| Code | Meaning |
|------|---------|
| 1 | Accessibility not trusted |
| 2 | App not found |
| 3 | Element not found |
| 4 | Action failed |
| 5 | Other error |
| 6 | Set attribute error |
| 7 | Screen capture not trusted |

## Agent skill

`skills/ax-cli/SKILL.md` teaches agents how to *use* the `ax` tool for UI inspection. It is not development guidance — it is a consumable skill for AI agents performing accessibility automation.

## Release

Releases are triggered by publishing a GitHub Release. CI builds signed/notarized universal binaries (x86_64 + aarch64 via `lipo`), uploads tarballs + SHA256 checksums, and auto-updates the `watzon/homebrew-ax` tap. Do not manually publish release assets.
