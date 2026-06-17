# BRIEF — Stone 275.2: reorder (core first), clean `deporder`, wire the enforcement gate

## The work (one paragraph)

Three things, all small: (1) **lift `wat/core.wat` to position 0** in `STDLIB_FILES` (`src/stdlib.rs`)
— this clears all 11 load-order violations `deporder` found (every one points at `core.wat@26`), and it
is safe because `core` has no eval-deps on any other file (its only outward refs are `:wat::Record::def`,
a defmacro = order-free, and `:wat::holon::HolonAST`, a builtin). Lift only `core`; preserve the relative
order of every other entry (they already satisfy their deps). (2) **Clean `wat/deporder.wat`'s nested-`if`
ladders** — `is-def-head?` (a 13-deep `if`/`=` ladder, lines 69–85) and `structural?` (4-deep, 44–51) are
set memberships in disguise; replace them with `HashSet/contains?` over a set of the literals (or `cond`
if the set constructor reads worse — the ladder must die either way). (3) **Wire the enforcement gate** —
a permanent test that runs `(:wat::deporder::verify-stdlib)` and asserts the result is **empty**; the build
goes red the instant any future eval-dep violation appears. The doctrine comment naming the rule goes on
`STDLIB_FILES` (intueri's lane).

## Read in order (the rooms)

1. **`src/stdlib.rs:30-277`** — the `STDLIB_FILES` array. `core.wat` is currently the entry near line 237.
   Move that one `WatSource` block to be the FIRST element (index 0), before the holon modules. Leave
   every other entry in its current relative order. Add a module-doc comment above the array stating the
   rule: *"Foundational → derived. A file precedes another only if it has no eval-time dependency on it
   (defmacro refs are order-free — registered in the pre-pass). Enforced by `:wat::deporder::verify-stdlib`
   (see tests) — a violation is a red build."*
2. **`wat/deporder.wat:44-85`** — the two `if`-ladders to kill (`structural?`, `is-def-head?`). Worked
   reference for set usage: grep `HashSet/contains?` / `HashSet` constructor usage in `wat/` and `src/`
   (`grep -rn "HashSet" wat/ src/collection/`). wat has set literals (ast-kind `"set"`); confirm the
   cleanest constructor for a set of String literals and use it. `def-head-kind` (89–92) is already a
   single `if` — leave it.
3. **`tests/probe_arc275_verify_stdlib.rs`** — the existing probe that evaluates `verify-stdlib` and
   counts violations (it currently documents the pre-reorder count). Either flip it into the permanent
   gate (assert `== 0` after the reorder) or add a sibling permanent test `tests/test_stdlib_load_order.rs`
   that asserts `verify-stdlib` returns an empty vector. The gate must be a NORMAL test (runs on every
   `cargo test`), not `#[ignore]`.

## STOP triggers

1. **STOP-1** — after the reorder, `(:wat::deporder::verify-stdlib)` must return **0** violations. If it
   does NOT (a violation remains or a new one appears), STOP and report the exact violation — it means a
   real eval-dep we didn't see (do not shuffle blindly to silence it).
2. **STOP-2** — if lifting `core` to front breaks the stdlib freeze (any NEW test failure beyond the known
   pre-existing `test_run_string_entry_direct`), STOP and report which test + the error. That names a real
   eval-dep on a file now after `core`; report it, don't paper over it.
3. **STOP-3** — if the `HashSet` set-of-Strings constructor won't compile cleanly, fall back to `cond`
   (not back to the `if`-ladder) and note it.

## Expectations (the scorecard — fill on your own re-run)

| what | command | expected |
|---|---|---|
| reorder clears violations | run `verify-stdlib` (the gate test) | **0 violations** (was 11) |
| deporder still correct | `cargo test --release --test test` (the 4 deporder deftests) | all 4 green |
| no regression | `cargo test --release --test test` | 248 passed / 1 failed (pre-existing `test_run_string_entry_direct` ONLY) |
| `core` is first | inspect `STDLIB_FILES[0]` | `wat/core.wat` |
| `if`-ladders gone | read `deporder.wat` `is-def-head?` / `structural?` | `HashSet/contains?` or `cond`, no nested-`if` ladder |

## Blast radius

- EDIT `src/stdlib.rs` — move one `WatSource` block to index 0 + the doctrine comment. No other array
  changes.
- EDIT `wat/deporder.wat` — `structural?` + `is-def-head?` only.
- EDIT/ADD the enforcement test (`tests/probe_arc275_verify_stdlib.rs` flipped, or a new permanent test).
- Nothing else.

## Discipline

- **Do NOT spawn sub-agents.** Single executor.
- Build green; the stdlib freezes at startup so a bad reorder fails fast.
- Do NOT commit — the orchestrator weighs + commits.
- Return: the `verify-stdlib` result after the reorder (must be 0), the 4 deftest results, the full-suite
  pass/fail, the cleaned `is-def-head?`/`structural?` (paste them), and any STOP hit.
