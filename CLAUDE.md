# ax — Development Guide

## Build & Run

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo run -- list        # Run a command
cargo run -- tree --app Finder --depth 3
```

## Architecture

```
src/
  main.rs              Entry point, CLI dispatch, permission check
  cli.rs               Clap derive definitions (Commands, TargetArgs, etc.)
  error.rs             AxError enum with AX error code mapping
  ax/
    element.rs         CFType-to-string converter, element_at_position(), set_timeout()
    app.rs             App enumeration via NSWorkspace, find_app_by_name(), permission check
    attributes.rs      read_all_attributes(), read_basic_info(), attribute_names()
    actions.rs         list_actions(), perform_action()
    tree.rs            build_tree() with depth limiting, role filter pruning, AXWindows fallback
    observer.rs        AXObserver FFI for watch command, CFRunLoop integration
  output/
    tree_fmt.rs        Unicode box-drawing tree renderer
    json_fmt.rs        serde_json serialization
    plain_fmt.rs       Key-value and table formatters
```

## Key Patterns

- **Raw AX API access**: Uses `accessibility-sys` FFI directly (not the high-level `accessibility` crate methods) for most operations, because `AXUIElement::attribute()` requires `AXAttribute<T>` typed attributes which is limiting. The raw `AXUIElementCopyAttributeValue` with `CFType` return is more flexible.
- **CFType conversion**: `ax/element.rs::cftype_to_string()` is the central utility. It inspects `CFGetTypeID()` and dispatches to CFString, CFNumber, CFBoolean, CFArray, AXUIElement, and AXValue (CGPoint/CGSize/CGRect/CFRange).
- **Memory management**: Uses `wrap_under_create_rule` for Copy/Create functions (caller owns), `wrap_under_get_rule` for Get functions (borrowed). The `accessibility` crate's `TCFType` handles retain/release via Drop.
- **App enumeration**: Uses `objc` crate's `msg_send!` to call NSWorkspace.runningApplications directly, not the `cocoa` crate's higher-level API.

## Testing Notes

- **Accessibility permission required**: Most commands need the terminal to have accessibility permission (System Settings > Privacy & Security > Accessibility). `ax list` works without it.
- **No automated tests yet**: AX API calls require a live desktop session with permission. Unit tests could cover CFType conversion and output formatting.
- **Good test targets**: Finder (always running), SystemUIServer (menu bar).

## Warnings

The `objc` crate (v0.2) generates `unexpected_cfgs` warnings from its `msg_send!` and `class!` macros. These are suppressed with `#![allow(unexpected_cfgs)]` in main.rs. This is a known issue with the older objc crate.
