# iced_impeccable

Two things live here:

1. A Rust library that runs any iced 0.14 `Application` headless behind a
   control socket (Unix socket, Windows named pipe): offscreen hardware
   rendering, injected keyboard and mouse input, a widget inventory with
   addressable targets, waits, screenshots with crop/zoom/annotation, and
   video recording. Plus the `iced-impeccable` binary (`ctl` client, `sheet`
   contact sheets, `help`) and a demo app.
2. The `iced-impeccable` skill plugin (`plugin/`, `.claude-plugin/`), a
   companion to the `impeccable` design skill for iced desktop apps.

## Module map

| File | Contents |
|---|---|
| `src/lib.rs` | public API: `Remote`, `run`, `is_headless`, `app_args`, `cli` |
| `src/remote.rs` | `Remote` builder, argv dispatch (`ctl` / `--headless` / windowed) |
| `src/args.rs` | host flags, `app_args` stripping |
| `src/protocol.rs` | `Command`, request parser, `USAGE` table, option splitting |
| `src/keys.rs` | `Keystroke`, key names and aliases, keyboard events |
| `src/target.rs` | target grammar, widget inventory nodes, filtering, node lines, match errors |
| `src/host.rs` | headless loop generic over `iced::Program` |
| `src/channel.rs` | transport (Unix socket / named pipe), multi-line replies |
| `src/client.rs` | `ctl` client: connect retry, batch mode, path resolution |
| `src/image.rs` | PNG I/O, crop, zoom, annotation, pointer sprite, contact sheet |
| `src/cli.rs`, `src/bin/iced-impeccable.rs` | standalone binary |
| `examples/demo.rs` | demo app exercising every protocol path |
| `plugin/skills/iced-impeccable/` | `SKILL.md` and `reference/*.md` |

## Gate

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --example demo
```

Windows compile check: `cargo check --target x86_64-pc-windows-gnu --all-targets`.

## Headless rules

- Never drive the user's desktop: no xdotool, ydotool, Xvfb, desktop screenshots.
- Never run, signal or replace the user's installed binaries.
- Each instance gets its own socket (`/tmp/<app>-agent-N.sock`) and state dir.
- Start with `--headless --control <socket> &`, no sleep; `ctl` retries the
  connection for up to 5 s. Drive with batches (`ctl <socket> - <<'EOF'`).
- Locate with `tree` or `screenshot --annotate`, act with `tap`; never guess
  coordinates from pixels. Small details via `screenshot --crop .. --zoom N`.
- End every instance with `quit`.

The skill's `reference/verify.md` is the full version for app work.

## Smoke test

```sh
cargo build --example demo --bin iced-impeccable
target/debug/examples/demo --headless --control /tmp/iip-1.sock &
target/debug/iced-impeccable ctl /tmp/iip-1.sock - <<'EOF'
tree
tap #name
type Ada
tap --nth 2 Save
wait-for Saved Ada
screenshot --annotate /tmp/iip-annotated.png
quit
EOF
```

## Commits

`type(scope): summary`, one line, max 60 chars, imperative. Types: feat, fix,
docs, chore, refactor, test, style, perf.
