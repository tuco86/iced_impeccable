> **Additional context needed**: target window sizes, OS targets (Linux / Windows / macOS) and usage contexts (docked, tiled, full screen, HiDPI).

Adapt an existing iced design to a different context: another window class, scale factor, OS, or origin (a website, a mobile app). The trap is treating adaptation as scaling. The job is rethinking the experience for the new context, inside the conventions of [iced.md](iced.md); read it before planning if Setup hasn't already.

## Assess Adaptation Challenge

1. **Source context**: what was it designed for, and what assumptions did it make? (One window size? Touch-first? A browser with hover and URLs? A phone with a tab bar?)
2. **Target context**: which window class (compact 960x600, standard 1280x800, wide 1920x1080), scale factor (1 / 1.5 / 2), and posture (keyboard and mouse at a desk, tiled beside other windows)?
3. **What breaks**: navigation that doesn't fit, layouts that stretch instead of restructure, gestures or controls that don't exist on a desktop?

## Adaptation Strategies

### Window size classes

- **Restructure, don't stretch.** A standard layout stretched to 1920x1080 is the failure mode (a centred narrow column in a sea of background, or a form field 1500 px wide). Switch structure with `iced::widget::responsive`, by the available size of the region, not by the window alone.
- **Compact (960x600)**: one primary region plus a collapsible secondary one. Sidebars collapse to an icon rail or a toggled panel; toolbars drop labels before dropping actions; inspectors move into the main flow.
- **Standard (1280x800)**: the reference layout. Sidebar + content, or content + inspector.
- **Wide (1920x1080)**: use the width for a third region (detail beside list, inspector beside canvas), more columns in grids, or a larger working surface. Cap text and form width with `max_width`; wide is not an excuse for longer lines.
- **Panes**: where the user arranges regions, use `pane_grid` with sensible `min_size`; persist the split ratios.
- **Tiling and resizing are normal.** A window manager can hand you any size; layout driven by size classes handles that for free. Set `min_size` in the window settings to the compact class so nothing below it has to be designed.

### Scale factor

- Never hard-code pixel sizes that assume scale 1. Design in logical pixels with tokens; iced multiplies by the scale. Offer `.scale_factor(|state| state.ui_scale)` from a setting, and test 1 / 1.5 / 2.
- Raster assets ship at 2x (or are SVG); a blurry icon at scale 2 is a finding. Drawing in `canvas` uses logical coordinates.
- Text sizes stay on the 12-32 token scale; scaling is the user's lever, not a reason to shrink below 12.

### Platform to platform (Linux / Windows / macOS)

Translate idioms; never transplant them:

| Concern | Handling in iced |
|---|---|
| Command key | `Modifiers::command()` (Ctrl on Linux and Windows, Cmd on macOS) |
| Window controls and title | OS-drawn by default; custom decorations only for a reason, with working drag, resize, minimise/maximise |
| File dialogs | native dialog crate behind `is_headless()`; never an in-app imitation |
| Fonts | bundle the UI font via `.font(bytes)` and `default_font` so all OSes render the same |
| Menus | macOS expects a global menu; elsewhere a toolbar or in-window menu |
| Paths and config dirs | per-OS data/config directories, not `~/.app` assumptions |

### Web or mobile to desktop

Reconform, don't reflow. Replace the hamburger menu and bottom tab bar with a sidebar or top toolbar, hover-only affordances with visible controls plus tooltips, scroll-everything pages with panes and fixed regions, tap targets with 24x24 / 32x32 pointer targets plus full keyboard operation, URL navigation with an in-app history. Then treat the result to the full [iced.md](iced.md); its slop test is the acceptance bar.

## Implement & Verify

- Drive structure from **size classes** via `responsive`, never from an OS or device check.
- Set `min_size` to the compact class; keep every control reachable there.
- Verify with the capture set in [verify.md](verify.md): standard dark and light, compact, wide, and hidpi (scale 2). Reach the same app state in each, `wait-idle`, `screenshot`, then `iced-impeccable sheet` to compare at a glance. Fix everything the round shows in one batch, confirm once.
- Keyboard and pointer behaviour are checked on the headless instance (`key tab`, `focused`, `tap`); real-OS behaviour (native dialogs, IME, window-manager integration) needs a human on that OS. Say which one produced the evidence.

When the adaptation feels native to each context, hand off to `/impeccable polish` for the final pass.

**NEVER**:
- Ship a stretched standard layout at the wide class
- Port one platform's, the web's or mobile's navigation onto the desktop
- Hide core functionality at the compact class (if it matters, make it work)
- Lock the window size to dodge a layout bug
- Hard-code pixel layouts that assume scale 1
- Trust headless captures alone for OS integration (dialogs, IME, window management)
