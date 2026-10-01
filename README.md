# iced_impeccable

Headless remote control for [iced](https://iced.rs) 0.14 applications, built
for coding agents that need to see and operate the UI they change, plus a
companion skill that brings the `impeccable` design workflow to iced desktop
apps.

The app runs offscreen with its real hardware renderer behind a local control
socket (a named pipe on Windows). A client sends one command per connection:
input, widget lookups by id or text, waits, screenshots. The app's own tasks
and subscriptions run unchanged.

## Integrate

```toml
[dependencies]
iced_impeccable = { path = "/home/hannes/work/git.doodleshnookie.net/tuco86/iced_impeccable", optional = true }
# or: { git = "ssh://git@git.doodleshnookie.net/tuco86/iced_impeccable.git", optional = true }

[features]
remote = ["dep:iced_impeccable"]
```

```rust
fn main() -> iced::Result {
    let app = iced::application(App::new, App::update, App::view)
        .subscription(App::subscription);

    #[cfg(feature = "remote")]
    return iced_impeccable::Remote::new(app)
        // periodic messages that mean "time passed", not activity
        .heartbeat(|m| matches!(m, Message::Tick(_) | Message::Frame(_)))
        // app-specific commands, e.g. `ctl <socket> reset`
        .command("reset", "clear the form", |_| Ok(Message::Reset))
        .run();

    #[cfg(not(feature = "remote"))]
    app.run()
}
```

- Give icon-only interactive widgets an id: `container(button(icon)).id("settings")`.
- Code that parses argv uses `iced_impeccable::app_args()` (argv without the host flags).
- Code that must not open OS dialogs checks `iced_impeccable::is_headless()`.
- Release builds never enable `remote`.

## Run

```sh
cargo build --features remote
target/debug/myapp --headless --control /tmp/myapp-1.sock [--size 1280x800] [--scale 2] \
    [--appearance dark|light] [--backend wgpu|tiny-skia] &
target/debug/myapp ctl /tmp/myapp-1.sock info          # or: iced-impeccable ctl ...
iced-impeccable ctl /tmp/myapp-1.sock - <<'EOF'         # batch: one command per line
tree
tap #name
type Ada
tap --nth 2 Save
wait-for Saved Ada
wait-idle
screenshot --annotate shots/form.png
quit
EOF
```

`ctl [--wait MS] [--keep-going] <socket> <command...>` retries the connection
for up to 5 s (no sleep after starting the host), prints the reply, and exits
0 on `ok`, 1 on `err`, 2 on bad usage. With `-` it reads a batch from stdin
(blank lines and `#` comments skipped) and stops at the first `err` unless
`--keep-going`.

## Protocol

One request line per connection; the reply is everything the host writes
until it closes the connection. The first line starts with `ok` or `err`,
further lines are payload. Coordinates are UI-logical pixels; a PNG pixel is a
logical pixel times the scale.

| Command | Reply |
|---|---|
| `help` | `ok` + one line per command, targets, keys, coordinates, custom commands |
| `info` | `ok pid P app NAME size WxH scale S appearance dark\|light backend NAME` |
| `size` | `ok W H SCALE` |
| `move X Y [MS]`, `down [BUTTON]`, `up [BUTTON]`, `click X Y [BUTTON] [MODS]`, `dblclick X Y`, `drag X1 Y1 X2 Y2 [STEPS] [MS]` | `ok` |
| `scroll X Y DY [DX]` | `ok`; wheel lines, DY>0 scrolls up, DY<0 scrolls down |
| `key SPEC`, `keydown SPEC`, `keyup SPEC`, `type TEXT` | `ok` |
| `tap [--nth N] TARGET` | `ok X Y` (clicked point) |
| `find [--nth N] TARGET` | `ok X Y W H` (visible bounds) |
| `find-all TARGET` | `ok N` + one `X Y W H` line per match (` hidden` when scrolled away) |
| `focused` | `ok X Y W H` or `err none` |
| `tree [--all]` | `ok N` + node lines |
| `wait-for [--timeout MS] TARGET` | `ok X Y W H` or `err timeout` (default 5000) |
| `wait-gone [--timeout MS] TARGET` | `ok` or `err timeout` (default 5000) |
| `wait-idle [MS]` | `ok idle`, `ok animating` or `err timeout` (default MS 200) |
| `screenshot [--annotate] [--crop X Y W H] [--zoom N] PATH` | `ok PATH WxH scale S` (+ node lines with `--annotate`) |
| `record PATH`, `record-stop` | `ok recording PATH WxH`, `ok PATH N frames Ts` (needs ffmpeg) |
| `resize W H` | `ok W H` (clamped to the window's min/max size) |
| `scale F` | `ok` |
| `appearance dark\|light` | `ok` |
| `clip`, `clip-set TEXT` | `ok TEXT`, `ok` |
| `quit` | `ok`, or `ok forced` when the app ignored the close request for 5 s |
| `<custom> [ARGS]` | `ok`, `err NAME: ...`, or `err unknown command "NAME" (see help)` |

**Targets**: `#ID` (widget id), `~SUBSTRING` (text contains), otherwise exact
text. `--nth N` picks the Nth match (1-based, tree order). Errors say what to
do next: `err not found: Sav; similar: "Save", "Saved Ada"`,
`err ambiguous: 2 matches (use --nth): ...`,
`err not visible: X Y W H (scroll it into view)`.

**Node lines**: `{idx} {kind} at {cx},{cy} box {x} {y} {w} {h}` then optional
` id=ID`, ` focused`, ` hidden`, ` offset X,Y content WxH` (scrollable),
` text="..."`. Kinds: `text`, `input`, `focusable`, `scrollable`,
`container`, `custom`. Anonymous containers with content are left out unless
`--all`.

**Keys**: one character, a name (`enter`, `esc`, `tab`, `left`, `pgdn`, `f5`,
...), or `code:<Name>` for a physical key; modifiers `ctrl+`, `shift+`,
`alt+`, `super+` in any order.

**`wait-idle`**: `ok idle` after MS without non-heartbeat activity and with no
frame pending; `ok animating` when only redraws kept it busy for another
second (a valid capture of a continuously animating UI, a finding at rest);
`err timeout` after 10 s of real activity.

## Contact sheets

```sh
iced-impeccable sheet [--cols N] [--labels a,b,c] [--max-width PX] OUT.png IN.png...
```

Lays screenshots out in a labelled grid; images are halved until the sheet
fits `--max-width` (default 2560).

## Skill

The `iced-impeccable` skill (in `plugin/`) layers iced platform guidance and
headless verification on top of the `impeccable` skill.

```sh
cargo install --path . --bin iced-impeccable --locked
omp plugin marketplace add /home/hannes/work/git.doodleshnookie.net/tuco86/iced_impeccable
omp plugin install iced-impeccable@iced-impeccable
```

## Demo

```sh
cargo run --example demo                                    # windowed
cargo run --example demo -- --headless --control /tmp/demo.sock &
cargo run --example demo -- ctl /tmp/demo.sock tree
```

## License

MIT OR Apache-2.0
