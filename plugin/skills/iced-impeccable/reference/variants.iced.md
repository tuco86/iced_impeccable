> **Replaces** impeccable's `live` and `generate` on iced: there is no browser to pick elements in. Variants are built into one binary, rendered headless, compared on a contact sheet, and the user chooses one.

Use for `live` and `generate [n] [action] [element]`. Verification mechanics are in [verify.md](verify.md); doctrine, modes and craft floor stay impeccable's.

## Flow

1. **Scope the element.** Name the view or style function being varied (a header, the form, a list row, a theme). Read the code and the current capture first. Decide the axis each variant explores; variants must differ in structure, hierarchy or colour logic, not in a nudged padding.
2. **Implement N variants in one build.** Default 3, maximum 4, labelled `a`, `b`, `c`, `d`. Put the switch in the affected view or style code, read once into a const-like value:
   ```rust
   fn variant() -> &'static str {
       static V: std::sync::OnceLock<String> = std::sync::OnceLock::new();
       V.get_or_init(|| std::env::var("IMPECCABLE_VARIANT").unwrap_or_else(|_| "a".into()))
   }
   // in the view or style fn:
   match variant() { "b" => header_b(state), "c" => header_c(state), _ => header_a(state) }
   ```
   Variant `a` is the current design when refining, so the sheet shows before/after. Variants share the data model; only presentation differs. Keep variant code in separate functions so the cleanup is mechanical.
3. **Build once**: `CARGO_TARGET_DIR=target/agent cargo build --features remote`.
4. **One headless instance per variant**, each with its own socket and state dir:
   ```sh
   IMPECCABLE_VARIANT=a target/agent/debug/<app> --headless --control /tmp/<app>-agent-a.sock --size 1280x800 &
   IMPECCABLE_VARIANT=b target/agent/debug/<app> --headless --control /tmp/<app>-agent-b.sock --size 1280x800 &
   IMPECCABLE_VARIANT=c target/agent/debug/<app> --headless --control /tmp/<app>-agent-c.sock --size 1280x800 &
   ```
   Give each its own state dir (`/tmp/<app>-agent-a/...`) as in verify.md.
5. **Drive each to the same state** with an identical batch (only the socket differs), then capture:
   ```sh
   for v in a b c; do
   iced-impeccable ctl /tmp/<app>-agent-$v.sock - <<EOF
   # ... same steps for every variant ...
   wait-idle
   screenshot /tmp/<app>-variants/$v.png
   quit
   EOF
   done
   ```
   The heredoc is unquoted so `$v` expands per variant; keep other `$` out of the batch lines.
6. **Contact sheet**:
   ```sh
   iced-impeccable sheet --labels a,b,c /tmp/<app>-variants/sheet.png /tmp/<app>-variants/a.png /tmp/<app>-variants/b.png /tmp/<app>-variants/c.png
   ```
   Use `--crop X Y W H --zoom N` on `screenshot` when the varied element is small, so the sheet shows it large. Add a light-appearance row if the variants differ by colour logic.
7. **Show the sheet to the user and ask which variant to keep** (impeccable's question tool). Say in one line what each variant explores. One round only; if none fits, ask what to change and run one more round, no more.
8. **Keep the chosen code.** Delete the `IMPECCABLE_VARIANT` switch, the `variant()` helper and the unchosen variants' functions; the app builds and behaves as if the chosen design had always been the only one. `quit` every instance and delete `/tmp` captures you created.

## Rules

- Variants obey [iced.md](iced.md) and craft-floor like any other work; "variant" is not licence for the slop list.
- Do not leave the switch in committed code. No env lookup survives step 8.
- Never drive the user's desktop or reuse their installed binary.
- The sheet is a decision aid, not a review capture; final verification still uses the capture set in verify.md.
