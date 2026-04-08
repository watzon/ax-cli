# ax

A CLI alternative to macOS Accessibility Inspector. Inspect running applications' UI hierarchies, query element attributes, perform actions, and monitor accessibility events -- all from the terminal.

## Requirements

- macOS (uses the Accessibility framework)
- Rust 1.70+
- **Accessibility permission** must be granted to your terminal app

### Granting Accessibility Permission

1. Open **System Settings > Privacy & Security > Accessibility**
2. Click the **+** button
3. Add your terminal application (e.g., Terminal.app, iTerm2, Ghostty, Alacritty)
4. Restart your terminal

Without this permission, commands that query UI elements will fail with a clear error message. The `ax list` command works without permission since it uses NSWorkspace.

## Installation

### Homebrew

```bash
brew tap watzon/ax
brew install ax
```

### From source

```bash
git clone https://github.com/watzon/ax-cli.git
cd ax-cli
cargo install --path .
```

## Agent Skill

An agent skill is available at `./skills/ax-cli` that teaches AI agents how to use `ax` effectively for UI inspection tasks. This skill works with any agent framework that supports the skills format.

**Install by copying:**
```bash
cp -r skills/ax-cli ~/.claude/skills/
```

**Or install via npx:**
```bash
npx skills install https://github.com/watzon/ax-cli --skill ax-cli
```

Once installed, agents can use `ax` to inspect application UI trees, query attributes, and assist with accessibility-related development tasks.

## Usage

### List running applications

```bash
ax list
ax list --json
```

```
NAME                 PID  BUNDLE ID
Calculator          1234  com.apple.calculator
Finder              1979  com.apple.finder
Safari             12345  com.apple.Safari
```

### Inspect an element

Inspect the application element itself, or target a specific element with `--focused` or `--point`:

```bash
ax inspect --app Finder
ax inspect --app Safari --focused
ax inspect --point 500,300
ax inspect --app Finder --json
```

Shows basic info (role, title, value, position, size), all advanced attributes, and available actions.

### Print accessibility tree

```bash
ax tree --app Finder
ax tree --app Finder --depth 3
ax tree --app Safari --filter button
ax tree --app Safari --extras          # include frame (position/size) and URLs
ax tree --app Safari --visible         # only elements visible in the window viewport
ax tree --pid 1234 --json
```

```
(AXApplication) [AXStandardWindow]
├── "Finder" (AXWindow) [AXStandardWindow] @(0,38 1440x862)
│   ├── (AXGroup)
│   │   ├── (AXToolbar)
│   │   │   ├── "Back" (AXButton) @(60,52 32x32)
│   │   │   ├── "Forward" (AXButton) @(96,52 32x32)
│   │   │   └── "View" (AXPopUpButton) @(200,52 80x32)
│   │   └── (AXSplitGroup)
│   │       ├── (AXScrollArea)
│   │       └── (AXScrollArea)
│   └── (AXGroup)
```

- `--filter` keeps only branches containing elements whose role matches the filter, so `--filter button` shows all buttons and their ancestor containers.
- `--extras` (`-x`) adds frame data (screen position and size) and URLs (e.g., `AXURL` on link elements) to each node. In JSON mode, frames are structured as `{"x", "y", "width", "height"}` numeric fields.
- `--visible` filters the tree to only elements whose frames fall within the window's viewport, hiding offscreen/scrolled-away content. Implies `--extras`.

### List all attributes

```bash
ax attrs --app Finder
ax attrs --app Safari --focused
ax attrs --point 100,200 --json
```

### Perform an action

```bash
ax action AXPress --app Calculator --focused
ax action AXShowMenu --app Finder --point 500,300
```

Common actions: `AXPress`, `AXShowMenu`, `AXConfirm`, `AXCancel`, `AXIncrement`, `AXDecrement`, `AXRaise`.

### Watch for notifications

```bash
ax watch --app Finder
ax watch --app Safari --notification AXFocusedUIElementChanged
ax watch --app Finder --notification AXTitleChanged,AXValueChanged
ax watch --app Finder --json
```

Streams accessibility events to stdout. Use `--notification all` (default) for common notifications, or specify one or more comma-separated notification names. Press Ctrl+C to stop.

### Get element at screen coordinates

```bash
ax element-at 500 300
ax element-at 0 0 --json
```

## Global Options

| Flag | Description |
|------|-------------|
| `--json` | Output as JSON (shorthand for `--format json`) |
| `--format <plain\|tree\|json>` | Output format (default: plain) |
| `--no-color` | Disable colored output |

## Targeting Elements

Most commands accept these targeting flags:

| Flag | Description |
|------|-------------|
| `--app <name>` | Target app by name (case-insensitive substring match) |
| `--pid <pid>` | Target app by process ID |
| `--focused` | Target the focused element within the app |
| `--point <x,y>` | Target the element at screen coordinates |

## Building from Source

```bash
git clone https://github.com/watzon/ax-cli.git
cd ax-cli
cargo build --release
# Binary is at target/release/ax
```

## License

MIT
