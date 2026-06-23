# Arc 290 — re-sync the neglected workspace crates to current arcs

**Status:** SCOPED (2026-06-22). Surfaced by the arc-258 `-> :T` annihilation: the
codemod's universe-load (and `cargo test --no-fail-fast`) exposed that the workspace
member crates have **weeks of un-applied-arc drift** — they have been failing the
*current* checker but were never caught, because the habitual gate was
`cargo test --test test` (main crate only), which never loads them.

This is a known repeat (the crates were found neglected once before). The deeper fix is
not just clearing the drift but **closing the gate gap** so they can't silently re-drift.

## The neglected crates

`crates/wat-lru`, `crates/wat-holon-lru`, `crates/wat-telemetry`, `crates/wat-sqlite`,
`crates/wat-telemetry-sqlite` (+ `examples/with-lru`). Each has its own `tests/test.rs`
wat-loading harness, run only under `cargo test` (workspace `default-members`), not the
main-crate-only filter.

## The drift axes (grounded — counts from the `--no-fail-fast` workspace run)

1. **Type-keyword-as-value (arc-242 doctrine)** — `:wat::core::nil` (264) +
   `:wat::core::i64` (15) used in VALUE position → bare `nil` / an i64 literal. ONE
   position-aware codemod class (must spare legitimate type-position uses, e.g. fn-return
   `-> :wat::core::nil`). The dominant lever.
2. **`expect -> :T` inside spawned-program STRINGS** (~18, `<entry>` / `<spawn-process-program>`)
   — the AST codemod (`strip-arrow-ascription`) parses `.wat` and cannot see expect inside
   a program *string literal*. THE HARD ONE: needs a string-aware pass, or re-sourcing the
   spawned program from the (now-bare) files instead of an inline string.
3. **`:wat::core::define` retired → `:wat::core::defn`** (~7) — a boundary-aware rename
   (use `:wat::fix::rename-keyword-prefix` or the retirement-table teaching).
4. **`match -> :T`** — once arc-258 sub-strike 2 lands, codemod the crates' match sites via
   the generic `strip-arrow-ascription` (head-set `{:wat::core::match}`).
5. **Downstream (likely auto-resolve once 1–4 land):** `TypeMismatch` (179) + comm-position
   (8, send/recv must sit in match/expect scrutinee — they currently sit in malformed
   expects). Re-measure after the structural axes; only chase residue.

## Method

Per-axis probe → codemod → cascade (the arc-258 rhythm). Run codemods with the
**battery-disable** technique (`crates/wat-cli/src/bin/wat.rs` → core-only load) so the
still-drifted crates don't fail the checker at load while being rewritten — see the
BOOTSTRAP header in `wat/fix.wat`. Cast vigilia on the touched crate homes after.

## The gate gap (the real extirpation)

The drift accumulated because the routine gate didn't load the crates. **Close it:** make
`cargo test` (workspace) — or at least a crate-load smoke — part of the standard
strike gate, so a corpus-wide change can't pass while a crate is red. Bank the lesson
already recorded in arc-258: the gate is `cargo test` (default-members) +
`--no-fail-fast` (so a known lib floor doesn't mask later crate binaries), NOT
`cargo test --test test`.

## Done = the gate
`cargo test --no-fail-fast` green for `wat-lru` / `wat-holon-lru` / `wat-telemetry` /
`wat-sqlite` / `wat-telemetry-sqlite` / `examples/with-lru` (modulo the known main-crate
lib 36-floor), with the crate homes vigilia-clean.
