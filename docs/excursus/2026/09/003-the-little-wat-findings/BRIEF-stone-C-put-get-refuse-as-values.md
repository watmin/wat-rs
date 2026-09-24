# BRIEF — STONE C: `Lru/put` raises, `Lru/get` misses — no panic

Read `DESIGN-stone-C-put-get-refuse-as-values.md` first.

## The work

Replace the two `panic!`s in `src/rust_deps/cache.rs` (`put` ~`:151`, `get` ~`:163`). **`put`** with
an unhashable key raises a wat `RuntimeErrorKind::TypeMismatch` naming `:wat::cache::Lru/put`;
**`get`** with an unhashable key returns `None`. No signature visible to wat changes; no call site
moves. Update the module doc's failure-surface section and the arc-109 NOTE to record what shipped.

## Rooms

1. **`src/rust_deps/cache.rs:40-75`** (module doc — rewrite the "two guards panic" paragraph) and
   **`:140-172`** (`put`/`get`).
2. **`src/collection/eval.rs`** — the HashMap guards: `contains-key?` at ~`:203-207` (the miss) and
   the insert path's `TypeMismatch` (grep `value_is_key_hashable`). **Copy these.**
3. **`crates/wat-macros/src/codegen.rs:276`** — the generated dispatch fn returns
   `Result<Value, RuntimeError>`. ⚠ Find out whether a `#[wat_dispatch]` METHOD can itself return a
   `RuntimeError` that propagates as a raise. `sqlite.rs`'s `RawFault` is the OTHER mechanism (an Err
   VALUE in a wat `Result`) — **not** what is wanted here.
4. **`docs/arc/2026/04/109-kill-std/NOTE-the-cache-lru-panics-…md`** — record `put`/`get` shipped.

## Blast radius

`src/rust_deps/cache.rs` · the arc-109 NOTE · one new probe under `tests/diagnostics/`. If and only
if STOP-1 resolves that way, `crates/wat-macros/` — see STOP-1.

## STOP triggers

1. **A dispatch method cannot raise a `RuntimeError`.** STOP and report what the macro supports.
   ⛔ Do NOT fall back to returning a `Result` (that is the 56-site churn the DESIGN rejects) and do
   NOT keep a panic behind a nicer message.
2. **The raised error's `:location` names a `.rs` file** instead of the user's `.wat`. That is the
   F-006 family again. Report it with the verbatim `:location`; do not silently accept it.
3. **Any gate reddens that you did not add.** Capture whole, name the arm. ⛔ Do not re-run first.

## Prove it — a new probe, four cases

`put` direct · `put` via a generic `K` · `get` direct · `get` via a generic `K`, each keyed on an
`Lru` handle (the repros are in the DESIGN). Assert: no `panicked at`, no `RUST_BACKTRACE` on
stderr; `put` → a wat `TypeMismatch` naming `:wat::cache::Lru/put`; `get` → prints `None`/a miss.
**Mutation:** restore each `panic!` in turn — its cases go RED. Restore.

## Mechanics — ⛔ read

- The Bash tool caps at 600s. Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated
  foreground `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks (NOT
  `.floor/latest/clean.log` — it only appears at the end). Do not end your turn while it runs.
- Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — use `rustfmt <file>` or neither.
- `git add` BEFORE running `git ls-files`-based gates.
- A new `.rs`/`.wat` under `tests/` answers to `no_loose_string_assert`, `no_inlined_edn`,
  `no_inlined_wat_in_tests`, `every_tracked_wat_parses`, `every_wat_bad_fixture_actually_fails`.
