# iced platform

For desktop apps built with iced 0.14 (Rust) shipping to Linux, Windows and macOS: `iced::application(..)`, `iced::daemon(..)`, and anything built on `iced::Program`.

On desktop the visitor mode narrows what expression may override. Operability governs structure, navigation and interaction in every mode; brand expresses through the layer the toolkit leaves open (palette, type, spacing, motion, content). iced draws every pixel itself, so nothing is native "for free": platform conformance is something the code does, not something the toolkit supplies.

## The iced slop test

Would a power user trust this app after an hour of keyboard use, or does it look like the iced `todos` example with a new title? The tells:

- The stock `Theme::Dark` / `Theme::Light` palette with no brand decision behind it.
- Every button the default primary style; no secondary, text or danger hierarchy.
- Missing hover, pressed, focused or disabled states: style functions that ignore `Status`.
- Fixed-pixel layouts (`Length::Fixed(412.0)` everywhere) that break on resize or at scale 2.
- Mouse-only operation: no Tab order, no Enter-to-submit, no Escape to close.
- A centred narrow column floating in a wide window.
- Cards in cards; a `container` with a border around every group.
- Emoji or Unicode glyphs as icons.
- Web hamburger menus and bottom tab bars on a 1920x1080 window.

Default to plain built-in widgets styled consistently; depart only for a reason the user would thank you for.

## Layout & structure

- **Window size classes.** Design for three: compact 960x600, standard 1280x800, wide 1920x1080. Set a floor in the window settings: `window::Settings { min_size: Some(Size::new(960.0, 600.0)), size: Size::new(1280.0, 800.0), .. }` (or `.window_size(..)` / `.window(..)` on the builder).
- **Fill, don't fix.** Prefer `Length::Fill`, `Length::FillPortion(n)` and `Length::Shrink` over `Length::Fixed`. Fixed sizes are for icons, gutters and sidebar widths with a stated reason. Cap reading width with `max_width(..)`, not a fixed width.
- **Spacing tokens** as consts, one scale: `SP_1 = 4.0`, `SP_2 = 8.0`, `SP_3 = 12.0`, `SP_4 = 16.0`, `SP_6 = 24.0`, `SP_8 = 32.0`. Pass them to `.spacing(..)` and `.padding(..)`; no stray literals like `13.0`.
- **Switch structure with `responsive`.** `iced::widget::responsive(|size| if size.width < 1100.0 { compact(..) } else { standard(..) })` picks the layout by the real available size, not the window size, so it also works inside panes. Keep the state outside the closure.
- **Resizable regions use `pane_grid`.** Sidebars, inspectors and split editors the user must resize are a `pane_grid` with `min_size`; drag state lives in `pane_grid::State`.
- **Scrolling.** One scrollable per axis per region. No nested same-axis `scrollable`s; scrollables inside a `Shrink` column collapse silently. Put `.height(Length::Fill)` on the scrollable.
- **Alignment.** Use `align_x` / `align_y` and `Space::new().width(Length::Fill)` to pin toolbars, not computed pixel offsets.
- **Large titles do not exist.** Top-level views get a heading in the content area, not a title bar; the OS draws the window title.

## Pointer & keyboard targets

- **24x24 logical minimum** for every clickable control, **32x32** for primary actions. A 16 px icon sits inside a button with `padding` that reaches the minimum.
- **Every style function handles every state.** `button::Style` closures match on `button::Status::{Active, Hovered, Pressed, Disabled}`; `text_input::Status::{Active, Hovered, Focused { is_hovered }, Disabled}`; the same for `pick_list`, `checkbox`, `toggler`, `slider`. A style that returns one value for all statuses is a defect.
- **Visible focus.** Give focusable widgets (`text_input`, `text_editor`, custom focusables) a distinct `Border` / background for the focused state; `text_input` reports `Status::Focused`. iced 0.14 buttons take no keyboard focus at all: give every button action a keyboard path (shortcut, Enter on the form's input) instead.
- **Tab order.** Bind Tab and Shift+Tab with `iced::widget::operation::focus_next()` / `focus_previous()` (both return a `Task`) from a keyboard subscription (`iced::keyboard::listen()`, which yields the key events no widget captured). Order follows the widget tree; reorder the `view`, not the style.
- **Enter submits.** `text_input(..).on_submit(Message::Submit)`; the default button of a form gets the same message.
- **Escape closes** overlays, popups and modals; it never quits the app.
- **Shortcuts** for frequent actions (`Ctrl+S`, `Ctrl+F`, `F2`, `Delete`), shown in `tooltip` text. Use `Modifiers::command()` so macOS gets Cmd.
- **Right click** is `mouse_area(content).on_right_press(Message::Menu)`; context menus are an overlay built on the same pattern.
- **Cursor.** `mouse_area(..).interaction(mouse::Interaction::Pointer)` on custom clickable areas; the I-beam on text comes from `text_input`.

## Typography

- **Bundle the UI font.** The system font differs per OS and is absent headless; verification then shows a different face than the user sees. Load it with `.font(include_bytes!("../assets/Inter.ttf"))` and set `.settings(Settings { default_font: Font::with_name("Inter"), .. })` (or `.default_font(Font::with_name("Inter"))`). Bundle a bold or variable weight if the UI uses `Font { weight: Weight::Bold, .. }`.
- **Size tokens** as consts: 12 / 14 / 16 / 20 / 24 / 32. **12 is the floor**; body is 14 or 16. Pass them to `text(..).size(..)`; no inline literals.
- **`Shaping::Advanced`** (`text(..).shaping(text::Shaping::Advanced)`) for non-Latin scripts, ligatures and emoji; the default `Basic` shaping renders them wrong.
- **Aligned numbers.** Tables and counters use `Font::MONOSPACE`, or a font with tabular figures. Monospace is for code, data and measurement, not for a "technical" look.
- **Hierarchy by size and weight**, not by colour alone. Line height via `text::LineHeight::Relative(1.3)` for body.
- **Measure.** Body paragraphs 65-75 characters: `.max_width(..)` on the containing widget.

## Color & theme

- **Build a theme**, don't stay on a stock one: `Theme::custom("Brand", Palette { background, text, primary, success, warning, danger })` or `Theme::custom_with_fn(name, palette, |p| palette::Extended::generate(p))` when the generated extended palette needs overrides. Return it from `.theme(|_| ..)`; with no `.theme(..)` the app follows the system appearance.
- **Colours only through the theme.** Inside style functions read `theme.extended_palette()` (`background.base.color`, `background.weak`, `primary.strong`, `secondary.base.text`, `danger.base`, ...). No `Color::from_rgb(..)` / `color!(..)` outside the one `theme` module.
- **Light and dark are both first-class.** Follow the system unless the product pins one (write the pin into PRODUCT.md). Capture both (`appearance dark|light`).
- **Contrast.** Body and placeholder text >= 4.5:1, large text >= 3:1, UI boundaries (input borders, focus) >= 3:1. Check disabled states too; "disabled" is not "invisible".
- **Tinted neutrals.** Derive surfaces from the palette's `background.weak/strong` steps rather than hand-picked greys.
- **One accent.** `primary` drives interactive emphasis; decoration is not its job.

## Components & controls

- **Built-in widget per need**: `button`, `checkbox`, `radio`, `toggler`, `slider`, `pick_list`, `combo_box`, `text_input`, `text_editor`, `progress_bar`, `tooltip`, `table`, `scrollable`, `rule`, `markdown`. Reinventing these for flavour is the most common slop.
- **Modals** are a `stack![base, opaque(mouse_area(center(dialog)).on_press(Message::Close))]`: `opaque` swallows input for the layer below, the `mouse_area` closes on a click outside, `Escape` closes via the keyboard subscription. Dialogs have one default action and a visible Cancel. Use a modal only for a task that needs interruption or protected focus.
- **One icon set on one size grid** (for example 16 px glyphs from a single icon font or SVG set via `svg(Handle)`), one stroke weight. Icon-only buttons get a `tooltip` and an id (`container(button(..)).id("settings")`) so they are findable in verification.
- **Lists and settings** use plain rows with `rule` separators and a consistent label/control grid, not stacks of bordered cards.
- **Empty, loading and error states** are designed: an empty list says what to do next; an error names the problem and the recovery.
- **Selection and focus survive updates.** Re-rendering a list must not drop the focused `text_input`; keep widget ids stable.

## Motion

- **`iced::Animation`** (`Animation::new(false).duration(Duration::from_millis(180)).easing(animation::Easing::EaseOut)`) holds the value; read it in `view` with `animation.interpolate(from, to, now)`, update it in `update` with `animation.go_mut(target, now)`.
- **Frames only while animating.** Subscribe to `iced::window::frames()` only while `animation.is_animating(now)`; drop the subscription at rest. Mark the frame message as a heartbeat (`Remote::heartbeat`) so verification is not confused.
- **Duration 120-250 ms**, exponential ease-out from an already-visible default. One authored moment, not scattered effects.
- **Perpetual redraw at rest is a defect.** A spinner or clock that never stops costs battery and shows up in verification as `wait-idle` → `ok animating`. That reply is fine for a deliberate loading indicator and a finding for anything else.
- Honor reduced motion where the OS exposes it; otherwise offer a setting that sets durations to zero.

## Accessibility

iced 0.14 exposes no accessibility tree: screen readers cannot read the app. State that limit in findings instead of pretending otherwise, and make up for it where possible:

- **Full keyboard operation**: every action reachable via Tab / Shift+Tab, Enter, Space, arrows, Escape.
- **Visible focus** in both appearances.
- **Contrast** per the Color section, in both appearances.
- **Scale factors 1, 1.5 and 2**: use `.scale_factor(|_| ..)` from a user setting and check that nothing clips or overlaps (`scale 1.5` and `scale 2` in verification).
- **No colour-only meaning**: pair status colours with an icon or word.
- **Long and translated strings** wrap (`text` wraps by default; `Wrapping::None` must be deliberate) and do not push controls out of view.

## Craft floor on iced

Translation of impeccable's `craft-floor.md`:

- CSS units become tokens: `rem`/`px`/`ch` → `Pixels` consts and `max_width`; `clamp()` → size-class switches with `responsive`.
- Contrast numbers apply unchanged.
- Depth: `Shadow { color, offset: Vector::new(0.0, 2.0), blur_radius: 8.0 }` carries an offset and a soft blur; a zero-offset coloured halo is decoration. Prefer tonal layering through `background.weak/strong`.
- The bans on gradient text and on side-stripe accents apply to `Border` (a coloured border on one side is built from a thin `container` strip; do not) and `Background::Gradient` / `gradient::Linear`. Hard offset shadows only in a deliberately neobrutalist world.
- Glass, `backdrop-filter` and blur are not available in iced; do not fake them with translucent stacks as decoration.
- Nested cards remain wrong; a `container` with a border is a card.
- Emoji or Unicode glyphs as icons remain banned; use a real icon set.
- The browser-surface bullets (text selection, caret, custom scrollbars, underline offset) map to: `text_input::Style { selection, value, placeholder }`, `scrollable::Style` (rail, scroller), `text_editor::Style`, and `Font::MONOSPACE` numerals. Theme them from the palette; they are the cheapest signal that the app was built rather than assembled.
- States to verify: hover, pressed, focused, disabled, loading, error, empty, plus real content and keyboard focus.

## Verifying the build

Screenshots come from a headless instance of the app, never a browser and never the user's desktop. See [verify.md](verify.md). Capture every window class the app ships to, in both appearances, plus scale 2; say which capture produced the evidence.
