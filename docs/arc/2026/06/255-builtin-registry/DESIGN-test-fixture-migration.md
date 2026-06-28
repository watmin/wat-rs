# Test-infra annihilation — the co-located `.wat` fixture migration recipe (the fleet brief)

> **Builder-directed (2026-06-27): "we fix the tests before we resume 293."** The ENTIRE test suite migrates
> from inlined-wat-strings → co-located `.wat` fixtures, so every test is `cargo wat`-runnable + fix-wat-able +
> lint-checkable. **Annihilation, NOT a ratchet** — there are no "blessed" violations; all **428** inlined-wat
> `.rs` files migrate, `tests/nursery/` dissolves (the 179-file map: `NURSERY-DISSOLUTION-MAP.md`), and a final
> lint fails on ANY new inlined-wat test. This doc is the pinned recipe + worked references the fleet copies.

## The scheme (codified — `feedback_test_wat_is_colocated_fixture`)
- **Co-locate:** `tests/<group>/<probe-basename>.wat` beside `<probe-basename>.rs`. Same basename, differ only in
  extension. `build.rs` globs `*.rs` only → the `.wat` sidecar is inert to the harness.
- **Slurp via `wat::freeze::startup_beside(file!())`** — derives `<probe>.wat` from the caller's `file!()`
  (`.rs`→`.wat`); zero path literal, rename-safe. (`startup_from_file(rel)` = the explicit form for a fixture
  shared by >1 probe. Both in `src/freeze.rs`.) The probe then drives via `eval_in_frozen` of a specific call.
- **Test binaries are GROUPED:** one binary per `tests/<group>/` dir (e.g. `binary(lint)`), test fn by name —
  NOT one `[[test]]` per file. Run: `cargo nextest run --release -E 'binary(<group>)'` or `-E 'test(<fn>)'`.

## ⚡ Recipe finding (empirical, builder-surfaced 2026-06-27) — fixtures DO NOT need `:user::main`
The original `startup_from_source(...)` calls carried a filler `(:wat::core::defn :user::main [] -> :nil nil)`.
**It is NOT required** — `startup_beside` LOADS the fixture's definitions; it never auto-invokes main, and the
probe drives via `eval_in_frozen` of an explicit call. **Drop the `:user::main` line** — proven by removing it
and the probe still passing (`probe_arc277_lint_concat_abuse`, green without it). A fixture is just the `defn`s
the probe calls.

## The transform shapes (the breadcrumb's 5) → their co-located form
Per-file SHADOWDANCER JUDGMENT, not a blind codemod. The unit of the migration is **the inlined wat PROGRAM**
(scaffolding + data) → a fixture `defn`; the `.rs` shrinks to slurp-then-eval-then-assert.

| # | shape (HEAD) | co-located transform |
|---|---|---|
| 1 | **static** — a `const X = r#"…"#` wat program eval'd directly | move the program into the fixture as a `defn :t::run [] -> <ret>`; `.rs` eval_in_frozen's `(:t::run)` and asserts on the result. |
| 2 | **eval_in_frozen of a const** (most lint/types probes) | same as (1) — the fixture `defn` wraps the eval'd form; honest return type (read the verb's `-> :T`). |
| 3 | **multi-program-per-file** — several consts/cases in one `.rs` | enumerate each as a NAMED `defn` in the ONE co-located fixture (`:t::case-a`, `:t::case-b`, …); `.rs` calls each by name. |
| 4 | **`format!`-dynamic** — a `fn helper(body)` or `format!("…{X}…")` assembling the wat at runtime from per-case parts | the parameterized parts are themselves wat → **enumerate the concrete cases as named `defn`s** in the fixture (NOT pass strings as args — that re-inlines the wat). `.rs` calls `(:t::case-X)`. The rare GENUINELY-dynamic program (program built from non-wat runtime data) carries an explicit rune; lint probes are NOT that. |
| 5 | **`fn run(src)` helper** — a shared Rust helper threading a world + assertion | keep a thin Rust helper that takes the CALL string and does `startup_beside(file!())` + eval + extract; the wat lives in the fixture. (`file!()` inside the helper resolves to the helper's `.rs` → its sibling `.wat`.) |

## Worked reference — the `lint` pilot group (5 files, GREEN; commit this)
All 3 hard sub-shapes in one group, proven:
- **`probe_arc277_lint_concat_abuse` / `probe_arc277_lint_if_ladder`** — shape 2 (const+eval). Fixture `defn :t::lint
  [] -> :wat::core::Vector<wat::lint::Finding>`; `.rs` asserts the count.
- **`probe_arc277_1b_ladder_autofix`** — shape 2, returns `:wat::core::String`; `.rs` asserts substrings.
- **`probe_arc277_1c_concat_format_autofix`** — shapes 3+4+5 (was a `fn lint_fix(body)` helper + 2 cases). Fixture
  enumerates `:t::fix-bare` + `:t::fix-compound`; `.rs` keeps a thin `fn fix(call)` helper that slurps + evals.
- **`probe_arc277_1d_concat_fix_position_gate`** — shape 4 (const + `format!`). Fixture `:t::fix`; `.rs` calls it.

## The fleet plan (per-group, the gate is the judge)
1. **Pilot DONE: `lint` (5).** Recipe + worked references proven; full workspace gate floor 0.
2. **Fan out per-group**, smallest first to calibrate: `channel`/`diagnostics`/`program`/`value`/`reflection`/
   `function`/`comms`/`collection`/`process`/`services`/`wat_lang`/`macros`/`resolve`/`kernel`/`rete`/`types`.
   Each group's `.rs` files are one binary → independent; a sonnet shadowdancer per group, embedding this recipe +
   the worked references, returns the migrated files; the ORCHESTRATOR weighs (`binary(<group>)` green + full gate
   floor 0) before commit. **Standing discipline: NEVER worktrees** — sequence groups (or parallel agents that only
   WRITE, orchestrator builds), never parallel builds racing the same `target/`.
3. **`tests/nursery/` (179)** — dissolve per `NURSERY-DISSOLUTION-MAP.md`: each file moves to its domain group AND
   its inlined wat → fixture in one motion; then delete `tests/nursery/mod.rs` + `Cargo.toml:123-125` + `rmdir`.
4. **The absolute lint** — once 0 inlined-wat probes remain: a gate that FAILS on any new `startup_from_source(`-on-
   an-inline-string test (zero, not a ratchet). Legit dynamic-program uses (rare) carry an explicit rune.

## The gate (every group, non-negotiable)
- the group binary green: `cargo nextest run --release -E 'binary(<group>)'`
- NO real inlined wat left: `grep -rn 'startup_from_source(\|format!(' tests/<group> --include='*.rs'` → empty
  (a doc-comment mention of "format!" is fine — match the CALL `format!(` / `startup_from_source(`).
- whole workspace floor 0: `cargo nextest run --release` → `… passed, 0 failed` (SET-diff ∅ vs HEAD).

## Pairs
`CURRENT-STATE.md` (campaign breadcrumb) · `NURSERY-DISSOLUTION-MAP.md` (part B) ·
`feedback_test_wat_is_colocated_fixture` (the scheme) · `src/freeze.rs` (`startup_beside`/`startup_from_file`) ·
`tests/lint/*` (the worked references).
