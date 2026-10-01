# DESIGN.md and sidecar on iced

Used together with impeccable's `document.md` (and `extract.md`) for `document` and `extract` on an iced app. Impeccable's structure is unchanged: the eight canonical sections, the YAML frontmatter, and `.impeccable/design.json` (schemaVersion 2). This file only maps what the web-flavoured instructions look for onto iced code.

## Where the design lives (replaces document.md Step 1)

Search, in priority order:

1. The `theme` module: `Theme::custom(..)` / `Theme::custom_with_fn(..)` and the `Palette` they take; `palette::Extended` overrides.
2. Style functions: `fn primary(theme: &Theme, status: button::Status) -> button::Style` and siblings for `container`, `text_input`, `pick_list`, `checkbox`, `scrollable`, `rule`, and so on.
3. Constants: font bytes and `Font` values, size tokens (12 / 14 / 16 / 20 / 24 / 32), spacing tokens (4 / 8 / 12 / 16 / 24 / 32), radius and border consts, animation durations.
4. Window settings: `window::Settings { size, min_size, .. }` and `.default_font(..)`.
5. Views of the main components (button rows, forms, navigation, dialogs).
6. Rendered output: a headless capture set from [verify.md](verify.md) in place of "load the live site and sample computed styles". Read pixel values only from captures taken at scale 1.

## Token mapping

| DESIGN.md frontmatter | iced code |
|---|---|
| `colors` | `Palette` fields (`background`, `text`, `primary`, `success`, `warning`, `danger`) plus extended-palette steps (`background.weak/base/strong`, `primary.strong`, ...) in the `theme` module. Keep the key slugs descriptive; record in `colorMeta` which palette field or extended step each one is. |
| `typography` | font consts (`Font::with_name`, bundled bytes), size tokens as `f32` consts (`fontSize` in px), `text::LineHeight` for `lineHeight`, `Font { weight, .. }` for `fontWeight`. `letterSpacing` is not available in iced: write `"normal"`. |
| `rounded` | `border::Radius` consts (`RADIUS_SM`, `RADIUS_MD`) |
| `spacing` | spacing consts (`SP_1` .. `SP_8`) used in `.spacing(..)` and `.padding(..)` |
| `components` | style functions (`button::primary`, `container::card`, ...): `backgroundColor` / `textColor` from the `Status::Active` branch, `rounded` from `border.radius`, `padding` from the view that uses it |

State variants follow the web convention as sibling keys: `button-primary` / `button-primary-hovered` / `button-primary-pressed` / `button-primary-disabled` map to the `Status::{Active, Hovered, Pressed, Disabled}` branches. The focus ring (`text_input::Status::Focused`) and shadows do not fit the 8 component props; carry them in the sidecar.

## Sidecar mapping (`.impeccable/design.json`)

- `components[].html` holds the **Rust view snippet** for the component, for example `button(text("Save")).on_press(Message::Save).style(button::primary).padding([SP_2, SP_4])`. The live panel of the web flavour does not render it; it exists so agents reuse the exact construction.
- `components[].css` holds the **style function body**, for example `let p = theme.extended_palette(); match status { Status::Hovered => Style { background: Some(p.primary.strong.color.into()), .. }, .. }`. Keep it copy-pasteable.
- `components[].kind`: `button | input | nav | chip | card | custom` as usual.
- `extensions.shadows`: `Shadow { color, offset, blur_radius }` values with their role. `extensions.motion`: `Animation` durations and easings (for example `{ name: "standard", value: "180ms ease-out" }`).
- `extensions.breakpoints` = the three size classes as `{name, value: "WxH"}`: `compact` `960x600`, `standard` `1280x800`, `wide` `1920x1080` (adjust only if the app defines other classes).
- `extensions.colorMeta` / `typographyMeta`: as in document.md, `canonical` in the format the theme module uses (hex or `rgb`).
- `narrative`: as in document.md. State explicitly that depth is tonal layering or `Shadow`-based, whichever the code does; there is no glass or blur in iced.

## Running `document` on iced

- The scan is a **code scan** of the `theme` module and style functions (above), confirmed with captures; if no theme module exists, colours scattered as raw `Color::from_rgb` literals are themselves a finding to report, and the extraction names the repeated values.
- Never invent tokens the code does not use. Empty scales stay omitted.
- Existing `DESIGN.md`: follow document.md (show it first, ask refresh / overwrite / merge).
- `extract` pulls repeated raw values into the `theme` module and repeated widget construction into style functions or small view functions, then migrates every caller; no shims.
