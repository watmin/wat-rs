# BRIEF — Arc 277.1b: the nested-if-=-ladder auto-fix

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently.

## The work (one paragraph)

Give the report-only `nested-if-=-ladder` lint rule a real AUTO-FIX that rewrites the whole ladder into
`(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "a" "b" "c") x)`. This needs: a typed
`FixEdit` record, `Finding.fix` changing from `String` to `Option<FixEdit>`, a `fix.wat` offset-math
primitive, an applier in `lint.wat` that splices fixes via the existing `fix-text-apply`, and the ladder
rule computing the edit (using `ast-span` + the new `ast-end-span` for the form's extent).

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1b-ladder-autofix.md` § "The contract"** and
implement its five parts verbatim (FixEdit record; `Finding.fix : Option<FixEdit>`; `make-ladder-finding`
computes `Some(fe)` with `new-text` built via **format** — dogfood it; `fix-text-span-len` in fix.wat;
`apply-fixes` + `lint-fix-file` in lint.wat).

## Read in order (the rooms)

1. `docs/arc/2026/06/277-wat-lint-fix-fmt/DESIGN-277.1b-ladder-autofix.md` — THE SPEC.
2. `wat/lint.wat:39-46` — the `Finding` record (+ its doc comment ~25-38); add `FixEdit` beside it,
   change `fix` to `:wat::core::Option<:wat::lint::FixEdit>`, update the doc comment.
3. `wat/lint.wat:228-256` (`make-ladder-finding`) — add the FixEdit computation (`ast-span` +
   `ast-end-span` of `form`; `new-text` via `format`; `fix = (:wat::core::Some <fe>)`).
4. `wat/lint.wat:320-340` (`violation->finding`) + the `make-concat-finding` (277.1c) — change their
   trailing `""` to `(:wat::core::None)`.
5. `wat/fix.wat:148-161` (`fix-text-offset-of`) — model `fix-text-span-len` on it (two offset-of calls,
   subtract). Note the arg shape: `fix-text-offset-of` takes a `{:line,:col}` HashMap + `lines`.
6. `wat/fix.wat:283-318` (`fix-text-apply` + the `reverse`→apply pattern) — `apply-fixes` reuses these.
7. `wat/lint.wat:286-300` (`lint-file`) — `lint-fix-file [sf] -> String` = `(apply-fixes sf (lint-file sf))`.
8. `wat-tests/lint.wat` — add Case 7 (`ladder-autofix-rewrites` + a no-fix round-trip).
9. `tests/probe_arc277_1b_ladder_autofix.rs` — remove the `#[ignore = "arc 277.1b …"]` attribute.

## Implementation notes

- `Some`/`None`: `(:wat::core::Some <val>)` / `(:wat::core::None)` (see `wat/stream.wat` for Option use).
- `make-ladder-finding`'s `lits` are already the literal TEXTS (e.g. `"\"a\""`); `(:wat::core::string::join " " lits)` → `"a" "b" "c"`, valid HashSet elements. `var-name` is the bare symbol text.
- `apply-fixes`: extract `Some` fixes only; build `Tuple(off, old-len, new-text)` per the DESIGN;
  `(:wat::core::reverse edits)` then `(:wat::fix::fix-text-apply src rev-edits)`.
- Build the `{:line,:col}` maps for `fix-text-offset-of` from the FixEdit's start/end fields (mirror how
  `ast-span` returns `{:line N :col N}` — a HashMap of keyword→i64).

## STOP triggers (halt + report, do not improvise)
1. If changing `Finding.fix` to `Option<FixEdit>` breaks the existing lint deftests (Cases 1-6) in a way
   beyond mechanically passing `None` — STOP, report (the report-only rules must still produce findings;
   only the fix FIELD changed).
2. If the deporder gate goes non-zero (lint.wat→fix.wat dep) — STOP, report (fix.wat precedes lint.wat in
   stdlib.rs so it should be fine; if not, that's a real load-order find).
3. If `fix-text-apply` mangles anything OUTSIDE the ladder form (the rest of the source must be
   byte-identical) — STOP, report.

## Blast radius
`wat/lint.wat`, `wat/fix.wat`, `wat-tests/lint.wat`, `tests/probe_arc277_1b_ladder_autofix.rs`
(un-ignore) ONLY. No Rust changes. No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc277_1b_ladder_autofix      # 1/1 GREEN (rewrite to contains?)
cargo test --release -p wat --test probe_arc277_lint_if_ladder         # still GREEN (rule still reports)
cargo test --release -p wat --test probe_arc277_lint_concat_abuse      # still GREEN
cargo test --release --test test 2>&1 | grep "test result"             # deftest: 260 passed / 1 failed (was 259, +1 Case 7; the 1 = run_string_entry_direct)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result  # deporder: 1 passed / 0 failed
cargo test --release -p wat --lib 2>&1 | grep "test result"            # lib: 929 passed / 36 failed (UNCHANGED)
```
Report: a summary of each change, the six command outputs verbatim, any delta. Do not claim green you
did not see.
