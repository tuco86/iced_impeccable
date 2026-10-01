Run systematic **technical** quality checks on an iced desktop app and generate a comprehensive report. Don't fix issues; document them for other commands to address.

This is a code-level audit plus a headless check, not a design critique. Audit from the Rust source (views, style functions, subscriptions, theme module) and confirm behaviour on a headless instance per [verify.md](verify.md). No browser tooling and no `impeccable detect` applies. Score against [iced.md](iced.md), including its slop test; read it first if Setup hasn't already. The report skeleton mirrors impeccable's `audit.md` and `audit.native.md`; keep them in sync when changing it.

## Diagnostic Scan

Run comprehensive checks across 5 dimensions. Score each dimension 0-4 using the criteria below.

### 1. Accessibility

iced 0.14 exposes no accessibility tree, so screen-reader support is out of scope; say so once in the report and score what the app controls.

**Check for**:
- **Keyboard reachability**: start headless, then loop `key tab` + `focused` through the window (`focused` replies `ok X Y W H` or `err none`). Every focusable widget must be visited; count what was reached against `tree` nodes of kind `input` / `focusable`. iced 0.14 buttons take no keyboard focus (they report as `container`), so check that every button action also has a keyboard path (shortcut, Enter on the form's input). Unreachable actions, focus traps and focus lost after navigation are findings.
- **Visible focus**: `screenshot --crop X Y W H --zoom 3` on the focused widget in both appearances; a focused control that looks like an unfocused one fails.
- **Contrast**: text below 4.5:1 (large text 3:1), UI boundaries below 3:1, in dark and light (`appearance light`); disabled states included.
- **Scale**: layouts that clip or overlap at `scale 1.5` and `scale 2`; text under 12 logical px.
- **Pointer targets**: clickable controls under 24x24 logical (32x32 for primary actions); read the `box` sizes from `tree`.
- **Colour-only meaning** and missing text alternatives for icon-only buttons (no tooltip, no id).

**Score 0-4**: 0=Keyboard unusable, 1=Major gaps (controls unreachable, no focus indication), 2=Partial (reachable, but focus or contrast breaks), 3=Good (minor gaps), 4=Excellent (fully keyboard-operable, visible focus, contrast and scaling clean)

### 2. Performance

**Check for**:
- **Perpetual redraw at rest**: `wait-idle` on an idle screen replying `ok animating` means something requests frames forever (a `window::frames()` subscription that never drops, a `time::every` with a short period, an `Animation` never finishing). `err timeout` means the UI keeps changing.
- **Heavy work in `view`**: allocation-heavy list building, sorting, formatting, or parsing on every call; `view` runs on every update.
- **Missing caching**: custom drawing without `canvas::Cache` (cache cleared only on real change); expensive subtrees that could use `iced::widget::lazy` with a cheap dependency key.
- **Image handles rebuilt per view**: `image::Handle::from_bytes(..)` or `svg::Handle` created inside `view` instead of stored in state (each new handle defeats the GPU cache).
- **Blocking work in `update`**: file, network or CPU-heavy work run synchronously instead of in a `Task::perform` / `Task::future`; the UI freezes (visible as `wait-idle` and `tap` replies stalling).
- **Unvirtualized long lists**: thousands of rows in one `column!` inside a `scrollable`; page, window or virtualise them. Check the `Row 1..N` style hidden rows in `tree`: node count grows with content.
- **Startup**: heavy work before the first frame (`boot` / `new`) instead of a `Task` after it.

**Score 0-4**: 0=Janky or frozen, 1=Major problems (perpetual redraw, blocking `update`), 2=Partial, 3=Good (minor improvements possible), 4=Excellent (idle at rest, cached, async I/O, lean)

### 3. Appearance & Theming

**Check for**:
- **Raw colours outside the theme module**: `Color::from_rgb(..)`, `color!(..)`, hex literals in views and style functions instead of `theme.extended_palette()`.
- **Single appearance**: a pinned `Theme::Dark` with no light variant (or the reverse) when the product does not pin one; compare `desktop-standard-dark.png` and `desktop-standard-light.png`.
- **Stock palette**: unmodified `Theme::Dark` / `Theme::Light` / built-in themes with no brand decision (no `Theme::custom`).
- **Inconsistent style functions**: several near-identical button/container styles; style functions that ignore `Status`; radii, borders and shadows set inline instead of from tokens.
- **Token drift**: spacing, size and radius literals instead of the consts; fonts not bundled (`.font(..)`, `default_font`), so headless and other OSes differ.
- **Off-toolkit effects**: faked glass or blur, hard offset shadows, gradient text, side-stripe accents.

**Score 0-4**: 0=Hard-coded everything, 1=Minimal tokens, 2=Partial (theme exists, inconsistently used), 3=Good (minor raw values), 4=Excellent (palette-driven throughout, both appearances first-class)

### 4. Desktop Conformance (CRITICAL)

Score against [iced.md](iced.md), including its slop test. **Check for**:
- **Resize**: the window is resizable and `min_size` is set; `resize 960 600` and `resize 1920 1080` leave nothing clipped, overlapped or floating in empty space.
- **Shortcuts**: frequent actions have keyboard shortcuts, shown in tooltips; `Modifiers::command()` used so macOS gets Cmd.
- **Tab order**: `focus_next` / `focus_previous` wired; order matches reading order.
- **Enter and Escape**: forms submit on Enter (`on_submit`); Escape closes overlays and modals, never quits.
- **Right click**: context actions on rows and text via `mouse_area(..).on_right_press`.
- **Clipboard**: copy/paste work (`clip`, `clip-set` round-trip); selection-friendly text.
- **HiDPI**: crisp and unclipped at scale 2 (`desktop-hidpi-dark.png`); no pixel-snapped assumptions.
- **Close request with unsaved state**: the app handles `window::close_requests()` / `Event::Window(CloseRequested)` and guards data loss; `quit` replying `ok forced` after 5 s shows the app ignores or stalls on close.
- **Native file dialogs**: file open/save use the platform dialog (for example `rfd`) behind `is_headless()`; a missing dialog fallback blocks headless runs.
- **Web-shaped patterns**: hamburger menus, bottom tab bars, hover-only affordances, emoji icons, mixed icon sets.

**Score 0-4**: 0=Web port (nothing desktop), 1=Heavy violations (3-4 kinds), 2=Some (1-2 noticeable), 3=Mostly conformant (subtle issues), 4=Fully desktop-native in behaviour (a keyboard-fluent user trusts every screen)

### 5. Adaptivity

**Check for**:
- **Size classes**: compact 960x600, standard 1280x800, wide 1920x1080 each have a deliberate layout (`responsive`); the wide class is not a stretched standard layout, the compact class does not hide core functions.
- **Scale factors** 1 / 1.5 / 2 render without clipping or overlap.
- **Long and translated strings**: text 40% longer than English still wraps and keeps controls reachable; no truncation that hides meaning.
- **Fixed-pixel layouts** (`Length::Fixed` for content regions) that break on resize.
- **Pane behaviour**: `pane_grid` panes with sane `min_size`; sidebars collapse on compact.
- **Content extremes**: empty lists, one item, thousands of items, very long names.

**Score 0-4**: 0=One window size only, 1=Major breakage (compact or wide broken), 2=Partial, 3=Good (minor edge cases), 4=Excellent (adapts across size classes, scale factors and string lengths)

## Generate Report

### Audit Health Score

| # | Dimension | Score | Key Finding |
|---|-----------|-------|-------------|
| 1 | Accessibility | ? | [most critical issue or "--"] |
| 2 | Performance | ? | |
| 3 | Appearance & Theming | ? | |
| 4 | Desktop Conformance | ? | |
| 5 | Adaptivity | ? | |
| **Total** | | **??/20** | **[Rating band]** |

**Rating bands**: 18-20 Excellent (minor polish), 14-17 Good (address weak dimensions), 10-13 Acceptable (significant work needed), 6-9 Poor (major overhaul), 0-5 Critical (fundamental issues)

### Desktop Conformance Verdict
**Start here.** Pass/fail: does this read as a desktop app or a ported website / the stock iced example? List specific violations. Be brutally honest. Name which captures or ctl commands produced the evidence.

### Executive Summary
- Audit Health Score: **??/20** ([rating band])
- Total issues found (count by severity: P0/P1/P2/P3)
- Top 3-5 critical issues
- Recommended next steps

### Detailed Findings by Severity

Tag every issue with **P0-P3 severity**:
- **P0 Blocking**: Prevents task completion. Fix immediately
- **P1 Major**: Significant difficulty or guideline violation. Fix before release
- **P2 Minor**: Annoyance, workaround exists. Fix in next pass
- **P3 Polish**: Nice-to-fix, no real user impact. Fix if time permits

For each issue, document:
- **[P?] Issue name**
- **Location**: View, file, line
- **Category**: Accessibility / Performance / Theming / Conformance / Adaptivity
- **Impact**: How it affects users
- **Guideline**: The [iced.md](iced.md) rule it violates (if applicable)
- **Recommendation**: How to fix it
- **Suggested command**: Which command to use (prefer: /impeccable adapt, /impeccable animate, /impeccable audit, /impeccable bolder, /impeccable clarify, /impeccable colorize, /impeccable critique, /impeccable delight, /impeccable distill, /impeccable document, /impeccable harden, /impeccable layout, /impeccable onboard, /impeccable optimize, /impeccable overdrive, /impeccable polish, /impeccable quieter, /impeccable shape, /impeccable typeset)

### Patterns & Systemic Issues

Identify recurring problems that indicate systemic gaps rather than one-off mistakes:
- "Raw `Color::from_rgb` appears in 15+ views, should use `extended_palette()`"
- "Icon-only buttons are below 24x24 and carry no tooltip or id throughout the toolbar"

### Positive Findings

Note what's working well: good practices to maintain and replicate.

## Recommended Actions

List recommended commands in priority order (P0 first, then P1, then P2):

1. **[P?] `/command-name`**: Brief description (specific context from audit findings)
2. **[P?] `/command-name`**: Brief description (specific context)

**Rules**: Only recommend commands from: /impeccable adapt, /impeccable animate, /impeccable audit, /impeccable bolder, /impeccable clarify, /impeccable colorize, /impeccable critique, /impeccable delight, /impeccable distill, /impeccable document, /impeccable harden, /impeccable layout, /impeccable onboard, /impeccable optimize, /impeccable overdrive, /impeccable polish, /impeccable quieter, /impeccable shape, /impeccable typeset. Map findings to the most appropriate command. End with `/impeccable polish` as the final step if any fixes were recommended.

After presenting the summary, tell the user:

> You can ask me to run these one at a time, all at once, or in any order you prefer.
>
> Re-run `/impeccable audit` after fixes to see your score improve.

**IMPORTANT**: Be thorough but actionable. Too many P3 issues creates noise. Focus on what actually matters.

**NEVER**:
- Report issues without explaining impact (why does this matter?)
- Provide generic recommendations (be specific and actionable)
- Skip positive findings (celebrate what works)
- Forget to prioritize (everything can't be P0)
- Report false positives without verification
- Drive the user's desktop or a browser to gather evidence
- Run `impeccable detect` or hooks (not available on iced)
- Score screen-reader support the toolkit cannot provide
