# aurora_app

GUI-library-agnostic Aurora OS window management helpers.

Provides wrappers around `winit::window::Window` for setting Aurora-specific Wayland generic properties (status bar visibility, background, cover window linkage, etc.). This crate has no OpenGL or GUI dependencies — it only deals with window properties.

## Usage

```rust
use winit::event_loop::EventLoop;
use aurora_app::window::{MainWindow, CoverWindow};

let event_loop = EventLoop::new().unwrap();
let window = event_loop.create_window(Default::default()).unwrap();

// Wrap the window — this sets WINID and other main-window properties
let main_window = MainWindow::new(window);
main_window.set_statusbar_visible(true);

// Create and link a cover window
let cover_window = event_loop.create_window(Default::default()).unwrap();
let cover = CoverWindow::new(cover_window);
main_window.link_cover(&cover);
```

## Modules

- **`q_variant_compat`** — Serializes Rust primitives (`bool`, `u32`, `u64`, `String`) into Qt `QVariant` byte blobs so winit can set Aurora-specific window properties via `update_generic_property`.
- **`types`** — Shared data types. Currently contains `CoverAction` for cover-page action buttons.
- **`window`** — Window property wrappers:
  - `MainWindow` — wraps a winit window; manages status bar, background, and cover actions.
  - `CoverWindow` — wraps a winit window; sets `CATEGORY=cover` and `TRANSPARENT` properties.

## Key APIs

### `MainWindow`

| Method | Description |
|--------|-------------|
| `new(window: Window) -> Self` | Wrap a raw winit window; assigns a small sequential `WINID` |
| `window() -> &Window` | Access the underlying window |
| `window_id() -> u64` | The small sequential ID used for `WINID` and `__winref:` |
| `set_statusbar_visible(bool)` | Sets `STATUSBAR_VISIBLE` property |
| `set_cover_actions(app_id, &[CoverAction])` | Sets `_APP_COVER_ACTION` for the compositor |
| `link_cover(&CoverWindow)` | Links cover window to main window via `SAILFISH_COVER_WINDOW` |
| `into_inner() -> Window` | Unwrap the winit window |

### `CoverWindow`

| Method | Description |
|--------|-------------|
| `new(window: Window) -> Self` | Wrap a window with cover properties; assigns a small sequential `WINID` |
| `window() -> &Window` | Access the underlying window |
| `window_id() -> u64` | The small sequential ID used for `WINID` and `__winref:` |
| `set_transparent(bool)` | Sets `TRANSPARENT` property |
| `link_to_main(&MainWindow)` | Links this cover to the main window |

### `CoverAction`

```rust
pub struct CoverAction {
    pub id: String,
    pub icon: String,
}
```

Used with `MainWindow::set_cover_actions` to define cover-page buttons rendered by the Aurora OS compositor.

## License

Apache 2.0
