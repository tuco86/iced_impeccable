# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `drop PATH` command: delivers `FileHovered` then `FileDropped` for `PATH`;
  `ctl` resolves a relative path against its own working directory.
- `Remote::windowed` and `Remote::headless` setup hooks: run code only before
  the windowed app or only in a `--headless` process.

## [0.1.0] - 2026-10-02

### Added

- `Remote` builder and `run` for any iced 0.14 `Application`: `<bin> --headless
  --control <socket>` runs the app offscreen with a hardware renderer (wgpu,
  or tiny-skia via `--backend`), `<bin> ctl <socket> ...` drives it, anything
  else runs the windowed app. `heartbeat` marks periodic messages, `command`
  registers app-specific commands; `is_headless` and `app_args` for app code.
- Host flags `--size`, `--scale`, `--appearance dark|light`, `--backend`; the
  app's fonts, default font, theme, style, scale factor and window settings
  (`min_size`, `max_size`, `exit_on_close_request`) are honoured.
- Control protocol with multi-line replies: pointer and keyboard input (key
  aliases such as `left`, `esc`, `pgdn`), `tap` / `find` / `find-all` on
  targets (`#ID`, `~SUBSTRING`, exact text, `--nth N`) with errors that name
  similar texts, ambiguity and visibility, `tree` widget inventory, `focused`,
  `wait-for` / `wait-gone` / `wait-idle` (`ok idle`, `ok animating`),
  `screenshot` with `--annotate`, `--crop` and `--zoom`, `record` via ffmpeg,
  `resize`, `scale`, `appearance`, clipboard, `quit` that respects close
  requests, and `help`.
- Transport over a Unix socket, on Windows a named pipe.
- `iced-impeccable` binary: `ctl` with connect retry (`--wait`) and stdin batch
  mode (`-`, `--keep-going`), `sheet` contact sheets, `help`.
- Demo app (`examples/demo.rs`) exercising every protocol path.
- `iced-impeccable` skill plugin, a companion to the `impeccable` design
  skill: iced platform reference, headless verification, audit, adapt,
  variants and DESIGN.md mapping.

[Unreleased]: https://github.com/tuco86/iced_impeccable/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tuco86/iced_impeccable/releases/tag/v0.1.0
