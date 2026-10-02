# Headless verification

Every visual check of an iced app runs against a headless instance driven through a control socket by `iced-impeccable ctl`. The full command list is `iced-impeccable help`.

## Rules

- **Never drive the user's desktop.** No `xdotool`, `ydotool`, `Xvfb`, no desktop or window screenshots, no synthetic input to a real display.
- **Never run, signal or replace the user's installed binary** (for example `~/.cargo/bin/<app>`). Build your own into a separate target dir and run that.
- **Build**: `CARGO_TARGET_DIR=target/agent cargo build --features remote`. Release builds never enable `remote`.
- **One state dir per instance.** Point the app's data/config home at its own directory under `/tmp` (the app's environment variable or flag), so an instance never touches the user's data.
- **Signals by PID** taken from `info` (`ok pid <pid> ...`), never by process name. End every instance with `quit`.
- On Windows the control channel is a named pipe (`--control <name>`, no path); use the agent's scratch directory instead of `/tmp`, and run a copy of the `.exe`, since Windows locks a running executable.
- A UI change is checked the way a user meets it: the flow clicked through and screenshotted, not only compiled.

## Start

```sh
target/agent/debug/<app> --headless --control /tmp/<app>-agent-1.sock --size 1280x800 2>/tmp/<app>-agent-1.log &
```

A host started with a plain `&` dies when the shell that started it exits, and agent harnesses run each command in a fresh shell: the socket then never appears. Either start the host as a background job of the harness (a long-running/async command), detach it with `setsid <app> --headless ... </dev/null >/dev/null 2>/tmp/<app>-agent-1.log &`, or start it and run the first `ctl` batch in the same command. When `ctl` reports `No such file or directory` after 5 s, read the host's log before retrying.

Flags: `--size WxH` (default: the app's window size, else 1024x768), `--scale F` (OS scale, default 1), `--appearance dark|light` (default dark), `--backend wgpu|tiny-skia`. Other argv entries stay the app's own.

No readiness polling and no `sleep`: `ctl` retries the connection for up to 5 s (`ctl --wait MS <socket> ...` to change it).

## Drive

Use batches: one process, one command per line, replies printed in order; the batch stops at the first `err` and names the line on stderr (`--keep-going` runs everything).

```sh
iced-impeccable ctl /tmp/<app>-agent-1.sock - <<'EOF'
info
tree
tap #name
type Ada
tap --nth 2 Save
wait-for --timeout 3000 Saved
wait-idle
screenshot /tmp/<app>-shots/after-save.png
EOF
```

Lines starting with `#` and blank lines are skipped, so a batch can be commented.

### Locate, then act

- Find widgets with `tree` (node lines: `idx kind at cx,cy box x y w h id=.. focused hidden text="..."`) or `screenshot --annotate PATH`, which draws the numbered boxes and prints the node lines of what it drew. Coordinates come from `tree` / the legend, never guessed from pixels.
- Targets: `#ID` (widget id), `~SUBSTRING` (text contains), otherwise exact text; `--nth N` picks the Nth match (1-based, tree order) when a target is ambiguous. Pick lists, icon-only buttons and toggles report no text: give them an id in the app and `tap #id`. An empty `text_input` reports its placeholder as its text (`input ... text="Your name"`); address inputs by id.
- `tap [--nth N] TARGET` clicks the centre of the visible bounds and replies `ok X Y`. `find` / `find-all` report bounds without clicking. `focused` reports the focused widget (`err none` when nothing has focus).
- Errors guide the next step: `err not found: Sav; similar: "Save", ...` (typo), `err ambiguous: 2 matches (use --nth): ...` (add `--nth` or use `#id`), `err not visible: ... (scroll it into view)` (use `scroll X Y DY` first).
- Scrolling: `scroll X Y DY [DX]` in wheel lines; `DY > 0` scrolls up, `DY < 0` scrolls down.
- Keyboard: `key SPEC` (`key tab`, `key shift+tab`, `key ctrl+s`, `key left`, `key esc`, `key enter`), `keydown` / `keyup` for held keys, `type TEXT` for text. Unknown key names answer with the full list.
- Mouse: `move`, `down`, `up`, `click X Y [BUTTON] [MODS]`, `dblclick`, `drag X1 Y1 X2 Y2 [STEPS] [MS]`.
- Clipboard: `clip` reads, `clip-set TEXT` writes.
- Files: `drop PATH` drags a file onto the window and drops it.
- App-specific commands registered by the app (`Remote::command`) are listed under `custom:` in `help`.

### Waiting

- After navigation or any asynchronous change: `wait-for [--timeout MS] TARGET` (default 5000; `err timeout` on failure) or `wait-gone`.
- Before every screenshot: `wait-idle [MS]`.
  - `ok idle`: nothing changed for the quiet period and no redraw is due.
  - `ok animating`: only redraws keep it busy. This is a valid capture of a continuously redrawing UI (spinner, clock). At rest it is a finding: an unnecessary perpetual redraw.
  - `err timeout`: the UI keeps changing; investigate.

### Screenshots

- `screenshot PATH` writes the full window PNG (`ok PATH WxH scale S`; pixels = logical x scale).
- `screenshot --annotate PATH` overlays numbered outlines for locating widgets; not for review captures.
- Small details: `screenshot --crop X Y W H --zoom 3 PATH` (crop in UI-logical coordinates, zoom 1-8). Never `magick`, `convert` or other external image tools.
- Appearance and scale on a running instance: `appearance dark|light`, `scale F`, `resize W H` (clamped to the window's min/max size; reply `ok W H`). `size` replies `ok W H SCALE`.

## Capture set for impeccable's bounded passes

Write to `.impeccable/review/` (gitignored):

| File | Window | Scale | Appearance |
|---|---|---|---|
| `desktop-standard-dark.png` | 1280x800 | 1 | dark |
| `desktop-standard-light.png` | 1280x800 | 1 | light |
| `desktop-compact-dark.png` | 960x600 | 1 | dark |
| `desktop-wide-dark.png` | 1920x1080 | 1 | dark |
| `desktop-hidpi-dark.png` | 1280x800 | 2 | dark |

One instance can produce all of them; reach the same app state first, then:

```sh
mkdir -p .impeccable/review
iced-impeccable ctl /tmp/<app>-agent-1.sock - <<'EOF'
# ... drive to the screen under review ...
wait-idle
screenshot .impeccable/review/desktop-standard-dark.png
appearance light
wait-idle
screenshot .impeccable/review/desktop-standard-light.png
appearance dark
resize 960 600
wait-idle
screenshot .impeccable/review/desktop-compact-dark.png
resize 1920 1080
wait-idle
screenshot .impeccable/review/desktop-wide-dark.png
resize 1280 800
scale 2
wait-idle
screenshot .impeccable/review/desktop-hidpi-dark.png
scale 1
quit
EOF
iced-impeccable sheet .impeccable/review/sheet.png .impeccable/review/desktop-*.png
```

`ctl` makes relative `screenshot` paths absolute against its own cwd, so run the batch from the project root.

1. Look at `sheet.png` first for the overall picture (labels are the file stems).
2. Then open each file once; do not loop on screenshots.
3. Pass these file names explicitly to `impeccable-finish-reviewer` as the desktop window classes.

Instances that must differ (for example a state that needs a restart) use `-agent-2.sock` with their own state dir. Variants use [variants.iced.md](variants.iced.md).

## End

Every instance ends with `quit` (`ok`, or `ok forced` when the app ignored the close request for 5 s). The socket file disappears; confirm the process is gone by PID if in doubt.
