# Arc 170 slice 3 — Gap A EXPECTATIONS (sonnet scorecard)

**One spawn.** Author the three keyword-reflection forms + migrate Layer 2.

## Independent prediction

**Runtime band:** 60-120 min sonnet. Three deliverables (2 runtime primitives + 1 macro form) + Layer 2 migration + verification.

**Hard cap:** 240 min. Kill via TaskStop if exceeded.

## Scorecard (10 rows; sonnet self-scores then orchestrator verifies)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::core::keyword/to-string` runtime primitive registered + dispatched | grep `keyword/to-string` in src/ + Rust unit test |
| B | `:wat::core::keyword/from-string` runtime primitive registered + dispatched | grep + Rust unit test |
| C | `keyword/to-string` returns text WITHOUT leading colon | unit test asserting `to-string(:foo) == "foo"` not `":foo"` |
| D | Round-trip `(from-string (to-string k)) = k` works for ≥ 3 sample keywords | unit test |
| E | `:wat::core::keyword/of` macro special form recognized in `expand_form` | grep + macro-expansion unit test |
| F | `keyword/of` correctly constructs multi-arg parametric text (commas, args sans-colon) | unit test with multi-arg case |
| G | Layer 2 macro in `wat/test.wat` uses `keyword/of` for channel types | grep `keyword/of` in run-hermetic-with-io macro body |
| H | T18 + T18b updated to pass inner element types; both still pass | `cargo test --release --test wat_arc170_program_contracts t18` shows 2 passed 0 failed |
| I | Workspace stays at 0 failed (2184 throughout) | full workspace cargo test |
| J | `cargo check --release` green | clean |

**10 rows.** All must pass.

## Implementation approach

Phase 1 (runtime primitives, Rust):
1. Find existing keyword-related runtime primitives in `src/runtime.rs` (eval_call dispatch table around line 3700+; existing keyword handling)
2. Add `eval_keyword_to_string` + `eval_keyword_from_string` functions
3. Register dispatch arms (mirror existing kernel-verb registrations)
4. Add type-check schemes in `src/check.rs`
5. Unit tests in the appropriate Rust test module

Phase 2 (macro special form, Rust):
1. Add handling in `src/macros.rs::expand_form` after child recursion, before generic List handling
2. Recognize `:wat::core::keyword/of` head; verify children are all keywords; construct new keyword text
3. Add `MacroError` variant if needed for "keyword/of: non-keyword child" / "keyword/of: zero args"
4. Macro-expansion unit test

Phase 3 (Layer 2 migration, wat):
1. Update `wat/test.wat` `run-hermetic-with-io` macro body to use `(:wat::core::keyword/of :wat::kernel::Receiver ~input-type)` and `(:wat::core::keyword/of :wat::kernel::Sender ~output-type)`
2. Update T18 + T18b in `tests/wat_arc170_program_contracts.rs` to pass inner element types (`:wat::core::i64` instead of `:wat::kernel::Receiver<wat::core::i64>`)
3. Verify cargo test passes

## What sonnet should produce

1. **Code changes:**
   - `src/runtime.rs` (or wherever keyword primitives live) — two new eval functions + dispatch arms
   - `src/check.rs` — type-check schemes for the two new verbs
   - `src/macros.rs` — `keyword/of` special-form handling in `expand_form`
   - `wat/test.wat` — Layer 2 macro body uses `keyword/of`
   - `tests/wat_arc170_program_contracts.rs` — T18/T18b take inner element types
   - Rust unit tests in appropriate modules
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-A-KEYWORD-REFLECTION.md` mirroring Phase D SCORE structure:
   - Scorecard verification
   - Implementation locations (where each piece landed)
   - Honest deltas (≥ 3 categories)
   - Files modified
   - What's next (Gap B: Sender/close)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT rename `:wat::core::keyword` to `:wat::core::Keyword` (deferred to arc 109 follow-up)
- Do NOT touch `:wat::test::run-hermetic` (Layer 1) macro or driver
- Do NOT touch `deftest` / `deftest-hermetic`
- Do NOT touch BareLegacy* walker / spawn.rs / Process<I,O> struct
- Do NOT use deferral language in SCORE
- If a substrate ordering issue surfaces (e.g., quasiquote evaluation order makes `keyword/of` see un-substituted unquotes), STOP and report

## Tools required

- Read / Edit / Bash (cargo, git, grep)
- Write for SCORE doc
- No Agent invocations (single-agent slice)

## Verification commands

```bash
# Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Phase 1 verification
grep -n "keyword/to-string\|keyword/from-string" src/runtime.rs src/check.rs

# Phase 2 verification
grep -n "keyword/of" src/macros.rs

# Phase 3 verification
grep -n "keyword/of" wat/test.wat
cargo test --release --test wat_arc170_program_contracts t18 2>&1 | tail -10

# Final workspace baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2184 passed / 0 failed
- Post Gap A: 2184+N passed / 0 failed (N = new Rust unit tests for the primitives; T18/T18b stay at 2 passing tests but with simplified surface)

## Honest delta categories (anticipated)

1. **Module location for the runtime primitives** — which file owns them; rationale
2. **Macro expansion ordering** — how `keyword/of` composes with enclosing quasiquote `~unquote`; verify the substitution-then-construction order works
3. **Phase 3 macro shape** — does `~input-type` substitute correctly INTO `(keyword/of :Receiver ~input-type)`; surface findings
4. **Anything unexpected** — surfaced during authorship
