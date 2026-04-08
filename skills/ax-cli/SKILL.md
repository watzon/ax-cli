---
name: ax-cli
description: >
  Use the `ax` CLI to inspect and interact with macOS applications via accessibility APIs.
  This skill should be used whenever the agent needs to: read what's on screen in a native
  macOS app, find UI elements by role or position, read text content from app windows,
  discover what buttons/controls are available, perform clicks or other UI actions
  programmatically, monitor UI changes in real time, or answer questions about what an
  application is currently displaying. Think of `ax` as a way to "see" and "interact with"
  any running macOS application from the terminal. Use this skill even if the user doesn't
  mention accessibility directly — any task involving reading or controlling macOS app UI
  should go through `ax`.
---

# ax — macOS Accessibility Inspector CLI

`ax` lets you inspect and interact with any running macOS application's UI from the terminal. It reads element hierarchies, attributes, text content, and can perform actions like clicking buttons — all via the macOS Accessibility framework.

The terminal running `ax` must have Accessibility permission (System Settings > Privacy & Security > Accessibility). If a command fails with a permission error, tell the user to add their terminal app there and restart it.

## Quick Reference

| Task                             | Command                                        |
| -------------------------------- | ---------------------------------------------- |
| See what apps are running        | `ax list`                                      |
| See an app's full UI tree        | `ax tree --app "App Name" --depth 5`           |
| Find specific element types      | `ax tree --app "App Name" --filter button`     |
| Tree with positions, sizes, URLs | `ax tree --app "App Name" --extras`             |
| Only visible (on-screen) elements| `ax tree --app "App Name" --visible`            |
| Read all attributes of an app    | `ax inspect --app "App Name"`                  |
| Read the focused element         | `ax inspect --app "App Name" --focused`        |
| Read element at screen coords    | `ax element-at 500 300`                        |
| Click a button or trigger action | `ax action AXPress --app "App Name" --focused` |
| Watch for UI changes             | `ax watch --app "App Name"`                    |
| Get machine-readable output      | Add `--json` to any command                    |

## How to Find Things

The typical workflow for locating a specific piece of information in an app:

1. **Start broad** — `ax tree --app "App Name" --depth 3` for top-level structure
2. **Go deeper** — increase `--depth`, or use `--filter` to narrow by role
3. **See what's visible** — `ax tree --app "App Name" --visible` to focus on what's currently on screen
4. **Search with grep** — `ax tree --app "App Name" --depth 20 --no-color | grep -i "search term"` is very effective
5. **Inspect specifics** — once you find the area, use `ax inspect` or `ax attrs` to get full details

To determine which elements are visible on screen, use `--visible`. To extract URLs from links (e.g., post permalinks on X/Twitter), use `--extras` or `--visible` with `--filter link`.

App names are matched as case-insensitive substrings, so `--app safari` matches "Safari" and `--app code` might match "VS Code". If the match is ambiguous, `ax` lists the candidates.

## Targeting Elements

Most commands accept these flags to target a specific element:

- `--app <name>` — target by app name (case-insensitive substring)
- `--pid <pid>` — target by process ID
- `--focused` — target the element with keyboard focus in that app
- `--point <x,y>` — target the element at screen coordinates

Without `--focused` or `--point`, commands operate on the application element itself.

## Commands

### ax list

Lists running applications (name, PID, bundle ID). Works without accessibility permission.

```
ax list
ax list --json
```

### ax tree

Prints the accessibility tree as an indented hierarchy with unicode box-drawing. Each node shows its role (AXButton, AXStaticText, AXGroup, etc.), title, value, and key attributes.

```
ax tree --app Finder --depth 5
ax tree --app Safari --depth 15 --filter button
ax tree --app Safari --extras              # include frame + URL data
ax tree --app Safari --visible             # only viewport-visible elements
ax tree --app Safari --visible --filter link  # visible links with URLs
ax tree --pid 1234 --json
```

- `--depth N` — how deep to traverse (default: 10). Start with 3-5, go higher for nested content.
- `--filter ROLE` — keep only branches containing elements whose role matches the substring. `--filter button` shows all AXButton elements and their ancestor containers.
- `--extras` (`-x`) — include frame data (screen position and size as `@(x,y wxh)`) and URLs (`-> https://...`) for each element. In JSON output, frames are structured as `{"x", "y", "width", "height"}` numeric fields, and URLs appear as a `"url"` string field. Use this to determine element positions and extract links (e.g., `AXURL` on `AXLink` elements).
- `--visible` — filter the tree to only elements whose frames fall within the window's visible area. Elements scrolled out of view or offscreen are pruned. Implies `--extras`. Useful for determining what the user can currently see.

Web content inside browsers is exposed as deeply nested elements. Use `--depth 15` or higher to reach text content in web apps like X/Twitter, Gmail, Slack, etc.

### ax inspect

Shows detailed info about a single element: basic properties (role, title, value, position, size), all advanced attributes, and available actions.

```
ax inspect --app Finder
ax inspect --app Safari --focused
ax inspect --point 500,300
ax inspect --app Finder --json
```

### ax attrs

Lists every attribute name and value for an element, including whether each is settable.

```
ax attrs --app Finder
ax attrs --app Safari --focused --json
```

### ax action

Performs an accessibility action on an element.

```
ax action AXPress --app Calculator --focused
ax action AXShowMenu --app Finder --point 500,300
```

Common actions:

| Action                        | What it does                         |
| ----------------------------- | ------------------------------------ |
| `AXPress`                     | Click/activate (buttons, menu items) |
| `AXShowMenu`                  | Open context menu                    |
| `AXConfirm`                   | Confirm a dialog                     |
| `AXCancel`                    | Cancel/dismiss a dialog              |
| `AXIncrement` / `AXDecrement` | Adjust sliders, steppers             |
| `AXRaise`                     | Bring window to front                |

### ax element-at

Returns info about the element at given screen coordinates. Useful when you know a position but not which app owns it.

```
ax element-at 500 300
ax element-at 0 0 --json
```

### ax watch

Streams accessibility events to stdout in real time. Press Ctrl+C to stop.

```
ax watch --app Finder
ax watch --app Safari -n AXFocusedUIElementChanged
ax watch --app Finder -n AXValueChanged,AXTitleChanged --json
```

- `-n all` (default) watches all common notifications
- `-n AXFocusedUIElementChanged` watches only focus changes
- Comma-separate multiple notification types

## Global Flags

- `--json` — structured JSON output. Always use when parsing programmatically.
- `--no-color` — disable terminal colors. Use when piping output or grepping.
- `--format plain|tree|json` — explicit format selection.

## Understanding AX Roles

Common roles in the tree:

| Role                           | What it is                                       |
| ------------------------------ | ------------------------------------------------ |
| `AXApplication`                | The application itself                           |
| `AXWindow`                     | A window                                         |
| `AXGroup`                      | A container — most UI is nested in groups        |
| `AXButton`                     | A clickable button                               |
| `AXStaticText`                 | Read-only text label                             |
| `AXTextField`                  | Editable text input                              |
| `AXTextArea`                   | Multi-line text input                            |
| `AXWebArea`                    | Web content region (in browsers)                 |
| `AXScrollArea`                 | Scrollable container                             |
| `AXToolbar`                    | Toolbar                                          |
| `AXMenuBar` / `AXMenuBarItem`  | Menu bar and its items                           |
| `AXRadioButton`                | Radio button (also used for browser tab buttons) |
| `AXCheckBox`                   | Checkbox                                         |
| `AXPopUpButton`                | Dropdown/popup button                            |
| `AXTable` / `AXRow` / `AXCell` | Table structures                                 |

## Tips

- **Web content is deep.** Browser-rendered content (X/Twitter, Gmail, Slack web) is often 10-20 levels deep. Use `--filter AXStaticText` or pipe through `grep` to find text.
- **Use `--visible` for viewport awareness.** It filters the tree to only what's on screen, so you can tell what the user is currently looking at without scanning the full tree.
- **Extract URLs from links.** `--extras` exposes `AXURL` on link elements. For example, X/Twitter timestamp links contain the post permalink URL — use `--visible --filter link --json` to get all visible links with their URLs.
- **Frame data for positioning.** With `--extras`, every element includes its screen coordinates and size. In JSON output, frames are numeric objects (`x`, `y`, `width`, `height`), ready for programmatic use.
- **JSON for scripting.** The JSON structure mirrors the tree hierarchy with `children` arrays, making it easy to traverse programmatically.
- **Elements can go stale.** If an app's UI changes between query and action, you may get "Invalid UI element" errors. Just retry.
- **Timeout is 5 seconds.** `ax` sets a 5-second timeout per element to avoid hanging on unresponsive apps.
- **ax list always works.** It doesn't need accessibility permission, so use it to discover app names and PIDs even before permission is granted.
