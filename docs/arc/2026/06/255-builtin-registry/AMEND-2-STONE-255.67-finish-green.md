# AMEND 2 — STONE 255.67: finish green (mechanisms A, B, C of the second STOP)

**Drawn 2026-09-28.** **Executor: a fresh Sonnet subagent.** Commit locally on `main`; **do not push**.

## Where the stone is (orchestrator-verified)

- K1 (`9d13e81a7`, `canonical_type_key`) is right: its floor `.floor/2026-09-28T05-45-45Z` is **6213/6213 green**.
- The full-corpus conversion (`fd04778e4`, 2,513 files) is idempotent, census `no STOP-8`, clippy clean. Its floor
  `.floor/2026-09-28T06-00-51Z` is **57 failed** (not re-run). `SCORE-STONE-255.67-cutover-2-types.md` and
  `.floor/2026-09-28T06-00-51Z/ARM.txt` hold the full evidence. Read both first.

## The work (the orchestrator's rulings on the three mechanisms)

- **B (23 tests), in scope as K1's own class:** a type spelling enters through a hand recognizer that matches only
  `WatAST::Keyword` and silently bails on a symbol: `src/declare/parse.rs:773-776`
  (`try_parse_user_variadic_def_fn_form`'s return type) and `src/function/parse.rs:829-834` (`parse_defclause_form`'s
  shared `-> :T`). Route both through the **one type door** (`parse_type_node` / K1's `canonical_type_key`), so a
  keyword or a symbol type reads the same. Then **search for every other recognizer of this shape**: a type slot read by
  matching `WatAST::Keyword` instead of calling the type parser. Report each one and route it through the same door.
- **A (~33 tests), golden recapture:** goldens that pin a diagnostic's `:col` / `:end`, where the fixture's text
  legitimately changed width (`:wat::core::` → `wat.type/`). Confirm per test that **only** positions moved (the
  error kind, message and names are unchanged), then recapture through the existing capture mode
  (`assert_edn_matches_file!`'s capture path; see `src/lib.rs:~359-380`). An inline literal is rewritten to the captured
  value. **List every recaptured golden in the SCORE** with its old and new position. Any golden where more than a
  position changed is **not** mechanism A: STOP on it.
- **C (1 test), investigate:** `rete::kernel::tests::where_tree_branch_differential`: a static corpus scan now finds 12
  more `where-*` axis shapes than the hardcoded `NON_UNIFORM` list. Find why. If the scan keys shapes by **spelling**
  (so the same shape in two spellings counts twice), cure it at the scan (K1's door). If the 12 are genuinely new
  shapes, **STOP** and report them.

Then the gates: the release floor (`scripts/floor.sh`), clippy, census `--diff` against `.census/2026-09-28T04-23-04Z.txt`,
delta. Append to `SCORE-STONE-255.67-cutover-2-types.md`: each mechanism's cure, the recaptured goldens, the other
keyword-only recognizers found, and every gate verbatim.

## STOP triggers (checked against the work list: each fires only outside A/B/C as ruled)

- **STOP-1:** a golden where more than a position changed.
- **STOP-2:** C's 12 shapes are genuinely new, not spelling duplicates.
- **STOP-3:** a floor red outside A/B/C. Do not re-run it; quote the block verbatim from `.floor/<stamp>/`.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. If this amendment
contradicts the code, the code wins: say so. Commit with `git add -- <paths>`. **Do not push.**
