---
name: iced-impeccable
description: Use when designing, redesigning, critiquing, auditing, polishing or otherwise improving the UI of an iced (Rust) desktop app, or when visually verifying an iced app headless (start it offscreen, inject keyboard and mouse input, take screenshots). Companion to the impeccable skill: same commands and design doctrine, with iced-native platform guidance and headless verification instead of a browser.
version: 0.1.0
user-invocable: true
argument-hint: "[impeccable command] [target] | verify | integrate"
license: MIT OR Apache-2.0
---

This skill layers iced (Rust, 0.14) desktop guidance on top of the `impeccable` skill. The design doctrine stays impeccable's; the platform (widgets, layout, input, verification) is iced's.

## Relationship

Read `skill://impeccable` first. Its principles, modes, commands and references apply, except where this skill overrides them. An iced app surface is **Operate** mode unless the brief says otherwise: scanability, consistency and desktop expectations outrank expression, and brand lives in precise details.

Impeccable's bounded-pass rule holds unchanged: build fully, inspect once with a batched capture round, fix everything in one batch, confirm at most once, stop.

## Setup

1. Run impeccable's `context` launcher exactly as its Setup says (`"<impeccable skill dir>/scripts/impeccable" context`, once per session, cwd at the user's project).
2. PRODUCT.md `## Platform` is `desktop` and `## Stack` records `iced 0.14 (Rust)`. Impeccable 4.4.0 knows only `web`, `ios`, `android`, `adaptive`, so the launcher prints `WARNING: PRODUCT.md's ## Platform value desktop is not recognized; treating the project as web` and resolves `"platform": null`. Expected: do not "fix" the field and do not surface it to the user. Handle the launcher's directives like this:
   - `MANUAL_DETECTOR_REQUIRED`: web-only; ignore it. Never run `impeccable detect` on an iced app.
   - `IMAGE_TOOLS`: ignore it for screenshots; crops, zooms and comparisons go through `screenshot --crop X Y W H --zoom N` and `iced-impeccable sheet` ([reference/verify.md](reference/verify.md)), never `magick` or `ffmpeg`.
   - `WORLD_DISCOVERY_REQUIRED`: the launcher does not recognise Rust views as a visual implementation (`"hasVisualImplementation": false`). An app with existing iced views has an incumbent implementation; the directive applies only to a new build or an explicit redesign, as it says.
   - `AUTONOMY_DIRECTIVE_CHECK`, `SUBAGENT_AUTHORIZATION` and any other platform-neutral directive (`CONTEXT_STALE`, `UPDATE_AVAILABLE`, `IMAGE_GEN_AVAILABLE`): follow as impeccable says.
3. Read [reference/iced.md](reference/iced.md).
4. Before any UI edit, read impeccable's `craft-floor.md`, then the "Craft floor on iced" section of [reference/iced.md](reference/iced.md), which translates it.
5. During impeccable `init`, write Platform `desktop` and Stack `iced 0.14 (Rust)`, and skip its live-mode configuration step.

If the platform value `desktop` is ever recognised natively by a later impeccable version, this Setup is the only place to change.

## Command overrides

| Command | Use |
|---|---|
| `audit` | [reference/audit.iced.md](reference/audit.iced.md) instead of impeccable's `audit.md` |
| `adapt` | [reference/adapt.iced.md](reference/adapt.iced.md) instead of impeccable's `adapt.md` |
| `live`, `generate` | [reference/variants.iced.md](reference/variants.iced.md) (contact-sheet variants, no browser picker) |
| `document`, `extract` | impeccable's `document.md` / `extract.md` plus [reference/design-md.iced.md](reference/design-md.iced.md) |
| `hooks`, `detect` | Not available on iced. Say so in one line and move on. |
| every other command | impeccable's own reference; translate its web examples (CSS, HTML, media queries) with [reference/iced.md](reference/iced.md) |

## Verification

Every visual check goes through [reference/verify.md](reference/verify.md): a headless instance of the app, driven through its control socket, screenshotted with `iced-impeccable ctl`. Never the user's desktop, never a browser, never external image tools.

`verify` as the argument runs that flow on the current app and reports what it saw.

## Integrate

Run when the user passes `integrate`, and whenever the target app does not yet support `--headless --control` (check: `Cargo.toml` has no `iced_impeccable` dependency).

1. `Cargo.toml` of the app:
   ```toml
   [dependencies]
   iced_impeccable = { git = "https://github.com/tuco86/iced_impeccable", optional = true }

   [features]
   remote = ["dep:iced_impeccable"]
   ```
   When a local checkout of iced_impeccable exists next to the app's repository, a `path` dependency on it works the same and picks up local changes.
2. Wrap `main`: build the `iced::application(...)` value, then
   ```rust
   #[cfg(feature = "remote")]
   return iced_impeccable::Remote::new(app)
       .heartbeat(|m| matches!(m, Message::Tick(_) | Message::Frame(_)))
       .run();
   #[cfg(not(feature = "remote"))]
   app.run()
   ```
3. Mark periodic subscription messages (`iced::time::every`, `iced::window::frames`) as heartbeats in the closure above, so `wait-idle` is not kept busy by them.
4. Give icon-only interactive widgets an id: `container(button(icon).on_press(..)).id("settings")`, then address them with `tap #settings`. Pick-list entries and other widgets that report no text are reached the same way or via `tree`.
5. App code that parses argv uses `iced_impeccable::app_args()` (argv without the host flags). App code that must not open OS dialogs (file pickers, message boxes) checks `iced_impeccable::is_headless()` and falls back.
6. App-specific actions the agent needs (reset state, restart, seed data) are registered with `.command("name", "summary", |args| Ok(Message::...))`.
7. Release builds never enable `remote`.

After integrating, verify with [reference/verify.md](reference/verify.md).
