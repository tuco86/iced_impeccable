#!/usr/bin/env bash
# End-to-end smoke test: runs the demo headless and drives it through `ctl`,
# checking the first reply line of every command.
#
#   scripts/smoke.sh [tiny-skia|wgpu]    default tiny-skia: needs no GPU
#
# Runs under Git Bash on Windows too, where the control channel is a named
# pipe instead of a socket file.
set -euo pipefail

backend=${1:-tiny-skia}
cd "$(dirname "$0")/.."
target=${CARGO_TARGET_DIR:-target}
cargo build -q --example demo --bin iced-impeccable

dir=$(mktemp -d)
if [[ ${OS:-} == Windows_NT ]]; then
    sock=iced-impeccable-smoke-$$
else
    sock=$dir/demo.sock
fi
"$target/debug/examples/demo" --headless --control "$sock" --size 1100x720 \
    --backend "$backend" 2>"$dir/host.log" &
host=$!
trap 'kill "$host" 2>/dev/null || true; rm -rf "$dir"' EXIT

failures=0
# expect PATTERN COMMAND...: the reply's first line must match PATTERN.
expect() {
    local pattern=$1
    shift
    local reply
    reply=$("$target/debug/iced-impeccable" ctl "$sock" "$@" || true)
    if grep -qE -- "$pattern" <<<"${reply%%$'\n'*}"; then
        echo "ok    $*"
    else
        echo "FAIL  $*"
        echo "      expected /$pattern/, got: ${reply%%$'\n'*}"
        failures=$((failures + 1))
    fi
}

expect "^ok pid [0-9]+ app demo size 1100x720 scale 1 appearance dark backend " info
expect "^ok [0-9]+$" tree
[[ $("$target/debug/iced-impeccable" ctl "$sock" tree) == *'hidden text="Row 60"'* ]] \
    || { echo "FAIL  tree lists Row 60 as hidden"; failures=$((failures + 1)); }
expect "^err ambiguous: 2 matches \(use --nth\)" find Save
expect "^ok [0-9]+ [0-9]+$" tap '#name'
expect "^ok$" type Ada
expect "^ok [0-9]+ [0-9]+$" tap --nth 2 Save
expect "^ok [0-9]+ [0-9]+ [0-9]+ [0-9]+$" wait-for Saved Ada
expect "^err not visible: " find Row 60
expect '^err not found: Sav; similar: "Save"' find Sav
expect "^ok [0-9]+ [0-9]+$" tap '#settings'
expect "^ok idle$" wait-idle
expect "^ok [0-9]+ [0-9]+$" tap Copy
expect "^ok Ada$" clip
expect "^ok [0-9]+ [0-9]+$" tap Spinner
expect "^ok animating$" wait-idle
expect "^ok [0-9]+ [0-9]+$" tap Spinner
expect "^ok$" key left
expect '^err unknown key "nokey"; keys: ' key nokey
expect "^ok$" reset
expect "^ok$" wait-gone Saved Ada
expect '^err unknown command "palette" \(see help\)$' palette
expect "^ok$" appearance light
expect "^ok .*/light.png 1100x720 scale 1$" screenshot --annotate "$dir/light.png"
expect "^ok .*/zoom.png 800x320 scale 1$" screenshot --crop 0 0 200 80 --zoom 4 "$dir/zoom.png"
expect "^ok$" scale 2
expect "^ok .*/hidpi.png 2200x1440 scale 2$" screenshot "$dir/hidpi.png"
expect "^ok 1100 720 2$" size
expect "^ok$" quit

wait "$host" || { echo "FAIL  host exit status $?"; failures=$((failures + 1)); }
sheet=$("$target/debug/iced-impeccable" sheet "$dir/sheet.png" "$dir/light.png" "$dir/zoom.png" || true)
[[ $sheet =~ ^ok\ .*/sheet.png\ [0-9]+x[0-9]+$ ]] && echo "ok    sheet" \
    || { echo "FAIL  sheet: $sheet"; failures=$((failures + 1)); }
[[ ! -e $sock ]] || { echo "FAIL  socket left behind"; failures=$((failures + 1)); }

if ((failures > 0)); then
    echo "$failures failure(s); host log:"
    cat "$dir/host.log"
    exit 1
fi
echo "smoke ($backend): all passed"
