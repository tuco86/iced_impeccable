# Releasing

Checklist for a release of the `iced_impeccable` crate and the
`iced-impeccable` skill plugin. Both carry the same version.

## 1. Pick the version

Under Cargo's 0.x semver rules a public-API break is a minor bump
(`0.1.x` -> `0.2.0`), anything else a patch bump. A protocol change that breaks
existing `ctl` scripts (renamed command, changed reply shape) counts as a
break. Check the Rust API against the last tag:

```sh
cargo semver-checks --baseline-rev "$(git tag --list 'v*' --sort=-v:refname | head -n1)"
```

## 2. Bump the version

- `Cargo.toml`: `version = "X.Y.Z"`, then `cargo update -p iced_impeccable`.
- `plugin/.claude-plugin/plugin.json`: `"version"`.
- `.claude-plugin/marketplace.json`: the plugin entry's `"version"`.
- `plugin/skills/iced-impeccable/SKILL.md`: frontmatter `version`.

## 3. Update the CHANGELOG

- Rename `## [Unreleased]` to `## [X.Y.Z] - YYYY-MM-DD` and add a fresh empty
  `## [Unreleased]` above it.
- Add `[X.Y.Z]: https://github.com/tuco86/iced_impeccable/releases/tag/vX.Y.Z`
  at the bottom.

## 4. Run every gate

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --example demo --bin iced-impeccable
scripts/smoke.sh
cargo check --target x86_64-pc-windows-gnu --all-targets
cargo deny --log-level error check
```

## 5. Commit, tag, push

```sh
git commit -am "chore(release): X.Y.Z"
git tag -a vX.Y.Z -m "Release X.Y.Z"
git push origin main vX.Y.Z    # origin pushes to doodleshnookie and GitHub
```

## 6. Install locally

```sh
cargo install --path . --bin iced-impeccable --locked
omp plugin marketplace update iced-impeccable    # re-copies the skill files
omp plugin install --force iced-impeccable@iced-impeccable
```
