# Arc 143 Slice 6 — Pre-handoff expectations

**Drafted 2026-05-02 (late evening)** in parallel with slice 3 sweep.
Replaces the prior obsolete EXPECTATIONS-SLICE-6 written for the
killed sweep before the substrate gaps surfaced.

**Brief:** `BRIEF-SLICE-6.md`
**Output:** 1-2 NEW files (`wat/runtime.wat`, `tests/wat_arc143_define_alias.rs`)
+ 1 modified file (`src/stdlib.rs`) + ~250-word report.

## Setup — workspace state pre-spawn

- Slices 1, 2, 3 shipped (verified by orchestrator). The substrate
  primitives exist:
  - `:wat::runtime::lookup-define / signature-of / body-of` (slice 1)
  - Computed unquote in defmacro bodies (slice 2)
  - `:wat::runtime::rename-callable-name / extract-arg-names` (slice 3)
- The macro expander supports `,(expr)` evaluating at expand-time when
  head is a Keyword (slice 2's heuristic).
- No new substrate work needed for this slice — pure userland wat.
- `wat/std/` is OFF LIMITS (arc 109 killing it). New macro lives in
  `wat/runtime.wat` (NEW top-level file).

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | NEW file `wat/runtime.wat` + NEW file `tests/wat_arc143_define_alias.rs` + MODIFIED `src/stdlib.rs` (1 entry). No other Rust changes. NO files in `wat/std/`. |
| 2 | `wat/runtime.wat` exists with header | File present at `wat/runtime.wat` with a short header comment naming the namespace + slice. |
| 3 | `:wat::runtime::define-alias` defmacro registered | The defmacro form in `wat/runtime.wat`. Two `:AST<wat::core::keyword>` parameters (alias-name, target-name) returning `:AST<wat::core::unit>`. |
| 4 | Macro body uses computed unquote | The body is a quasiquote that calls `signature-of`, `Option/expect`, `rename-callable-name`, `extract-arg-names` at expand-time via `,(expr)`. |
| 5 | `src/stdlib.rs` registration | New entry in stdlib.rs registering `wat/runtime.wat` similar to existing wat/test.wat / wat/console.wat entries. |
| 6 | Test file present | `tests/wat_arc143_define_alias.rs` with at least 2 tests (macro expansion + functional). 3rd test (error case) is bonus. |
| 7 | Macro expansion test passes | The expansion test verifies a `(:wat::runtime::define-alias :alias :foldl)` invocation expands to a `(:wat::core::define ...)` form (verifiable via macroexpand-1 OR by registering and using the alias). |
| 8 | **`cargo test --release --workspace`** | Exit non-zero only because of the 1 pre-existing arc 130 LRU failure. The 2-3 new define-alias tests pass. ZERO new regressions. **OR**: clean Mode B failure surfaces an FQDN gap or typing gap; sonnet stops + reports exactly what failed. |
| 9 | No `wat/std/` additions | Verifiable: `git diff --stat -- wat/std/` shows no changes. The `:wat::std::*` namespace is off-limits per arc 109. |
| 10 | Honest report | 250-word report includes: `wat/runtime.wat` content verbatim, `src/stdlib.rs` change, macro expansion verbatim, test file's test count, test totals, honest deltas (especially any FQDN gap surfacing). |

**Hard verdict:** all 10 must hold. Row 8 is load-bearing for runtime
correctness — Mode A or clean Mode B both count. Row 9 is load-bearing
for the namespace discipline.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | wat/runtime.wat: 20-50 LOC including header + defmacro body. Test file: 50-100 LOC. Total slice diff: 80-180 LOC. |
| 12 | Macro body shape matches brief | Quasiquote with computed unquote calls; `Option/expect` for the None case. No invented helpers. |
| 13 | `src/stdlib.rs` style match | Registration entry mirrors existing wat-file entries exactly. |
| 14 | Test file uses `wat::test!` macro convention | Tests follow the existing test-attribution pattern (per the wat::test! proc macro discovery). |

## Independent prediction

- **Most likely (~50%) — Mode A clean ship:** all substrate pieces are
  in place; macro composes them mechanically. Tests pass. Substrate-
  as-teacher cascade end-to-end demonstrated. ~10-15 min runtime.

- **Mode B — FQDN gap (~25%):** the substrate's bare-name type
  registry IS canonical (verified by orchestrator crawl), but the
  macro's emitted define passes through a code path that requires
  FQDN names. Clean diagnostic; opens slice 5a (FQDN rendering fix);
  slice 6 relands.

- **Mode B — type-checker special-case gap (~10%):** the macro body
  composes substrate primitives in a way the slice 1 + slice 3
  type-checker special-cases don't anticipate (e.g., nested
  Option<HolonAST> unwrap). Clean diagnostic; small extension;
  slice 6 relands.

- **Mode B — Option/expect signature mismatch (~8%):** the
  Option/expect special form has constraints the brief didn't
  fully capture (e.g., type annotation requirements). Sonnet
  surfaces; reland with sharper brief.

- **Mode B — quasiquote-arity surprise (~5%):** the computed-unquote
  paths inside nested unquote-splicing have an interaction the brief
  didn't predict. Reland with worked example.

- **Mode B — registration site issue (~2%):** stdlib.rs registration
  shape differs from the brief's assumption.

## Methodology

After sonnet returns:

1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — verify file scope.
4. Read `wat/runtime.wat` directly to verify the macro body.
5. Run `cargo test --release --workspace` locally.
6. Verify Mode A by re-running the new test file's tests; OR verify
   Mode B by reading sonnet's failure description.
7. Score; commit `SCORE-SLICE-6.md`.

If Mode A: prep slice 7 (apply :reduce/:foldl + arc 130 substrate
call-site updates).

If Mode B: open the appropriate sub-slice (5a for FQDN, 5b for type-
checker extension, etc.); ship it; reland slice 6.

## Why this slice matters

Slice 6 is the END-TO-END validation of arc 143's substrate-as-teacher
cascade. Slices 1+2+3 are the FOUNDATION; slice 6 is the FIRST
CONSUMER. If it ships clean, the whole reflection layer demonstrably
works.

Mode A → slice 7 → arc 130 unblocks → arc 109 v1 closes.

Mode B → diagnostic + sub-slice → slice 6 relands trivially.

Either path is calibration. The substrate-informed brief discipline
either ships clean or surfaces a precisely-named gap.
