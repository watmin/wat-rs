# BRIEF — Arc 277.1c-fix: the concat→format auto-fix (bare-symbol slots only)

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently.

## The work (one paragraph)

Give the report-only `concat-abuse` rule a real auto-fix that rewrites a `string::concat` of literals +
**bare-symbol** values into a `format` call — but ONLY when every value slot is a bare symbol (so the
placeholder name = the symbol, mechanically honest). A concat with any compound value slot stays
report-only (`fix = None`) — its naming is a judgment deferred to arc 278. Reuses the 277.1b FixEdit /
apply-fixes machinery; only `make-concat-finding` changes.

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1c-fix-concat-format.md` § "The fix"** and
implement `concat-format-fix [form] -> Option<FixEdit>` verbatim: eligibility (every non-literal arg is
`ast-kind "symbol"`; no literal contains `"`/`{`/`}`), build template + deduped kwargs by folding args
(literal → its `ast-name` text into the template; symbol → `{name}` into template + `:name name` kwarg),
emit `new-text = "(:wat::core::format \"<template>\" :a a :b b)"`, `fix = Some(FixEdit <span> new-text)`.
Ineligible → `None`. Wire it into `make-concat-finding` (the `fix` field, currently `None`).

## Read in order (the rooms)

1. `docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1c-fix-concat-format.md` — THE SPEC.
2. `wat/lint.wat` — `make-concat-finding` (the concat finding ctor; currently passes `(:wat::core::None)`
   for `fix`) + `make-ladder-finding` (277.1b — copy its `ast-span`+`ast-end-span` → `FixEdit` shape) +
   the `FixEdit` record + `apply-fixes`/`lint-fix-file` (all already shipped — reuse, don't rebuild).
3. `wat/core.wat` (the `format` macro doc ~506) — confirm the template grammar your `new-text` must
   produce: `(:wat::core::format "…{name}…" :name val …)`, named slots, every `{name}` has a `:name`.
4. The `format` macro's template extraction shows `ast-name` of a STRING node returns its inner text —
   use that to get each literal's content for the template.
5. `tests/probe_arc277_1c_concat_format_autofix.rs` — remove the two `#[ignore = "arc 277.1c-fix …"]`.

## Implementation notes
- Eligibility predicate first; if any value arg is not a `symbol`, or any literal has `"`/`{`/`}`, return
  `None` (report-only) — do NOT attempt naming.
- DEDUP kwargs: the same symbol name appears once in the kwargs (two `{a}` against one `:a a` is valid
  format). Preserve first-seen order.
- `new-text` is a plain String built by `string::concat`; the spliced `format` call's template is wrapped
  in `\"…\"`. The extent to replace = `ast-span`..`ast-end-span` of the concat form (same as the ladder).

## STOP triggers (halt + report)
1. If a compound-slot concat gets auto-fixed (it must stay report-only) — STOP, report.
2. If `format`'s strict check rejects your generated call (a `{name}` without `:name`, or an unused
   `:name`) — STOP; your template/kwargs are out of sync.
3. If the ladder auto-fix (277.1b) or concat-abuse report (277.1c) regress — STOP, report.

## Blast radius
`wat/lint.wat` (`concat-format-fix` + `make-concat-finding`'s `fix`) + `wat-tests/lint.wat` deftest +
un-ignore the probe. No Rust changes. No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc277_1c_concat_format_autofix    # 2/2 GREEN (bare-symbol fixes; compound untouched)
cargo test --release -p wat --test probe_arc277_1b_ladder_autofix           # still GREEN (ladder fix intact)
cargo test --release -p wat --test probe_arc277_lint_concat_abuse           # still GREEN (detection intact)
cargo test --release --test test 2>&1 | grep "test result"                  # deftest: 262 passed / 1 failed (was 261, +1 Case 8)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result       # deporder: 1 passed / 0 failed
cargo test --release -p wat --lib 2>&1 | grep "test result"                 # lib: 929 passed / 36 failed (UNCHANGED)
```
Report: `concat-format-fix` + the `make-concat-finding` change, the command outputs verbatim, any delta.
Do not claim green you did not see.
