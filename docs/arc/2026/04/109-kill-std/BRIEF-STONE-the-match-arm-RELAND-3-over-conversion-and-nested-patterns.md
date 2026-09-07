# BRIEF — RELAND 3: one over-conversion, one unreached position

> Floor: **5186 passed, 38 failed** (from 108). Read from `^ +Summary`, not a tail.
> The four probe rows still PASS. `#[ignore]` = 0. Row 4 (retired clause refused) holds.

## ⛔ CAUSE 1 — THE CODEMOD CONVERTED SOMETHING THAT WAS NEVER AN ARM

`wat/core.wat:1429`, inside `defmacro :wat::core::->>`'s `fn` body:

```clojure
;; was:  `(~@step ~a)          a quasiquote SPLICE, in a fn body, not a match
;; now:  `[~@ {:step step} ~a]
```

The recogniser saw a 3-element list whose head is `~@step` and read it as `(head binder body)`.
`->>`'s macro body then fails at eval — *"got wat::core::HashMap `{:step: <WatAST>}`"* — which is
13 of the 38 (all `thread_first`/`thread_last`/pipeline tests).

★ **This is the cost of the previous reland's own fix.** To catch arms that are SPLICED into a match
later (`~@serve-op-arms`), the recogniser was loosened off requiring a `match` parent. That loosening
is what lets it fire on `` `(~@step ~a) ``. **The two cannot both be solved by shape alone** — a
spliced arm and a spliced call are the same shape — so this needs a discriminator that is not
structure: the enclosing form, a marker, or an explicit path/site list for the spliced cases.

⚠ **Every previous STOP I wrote pointed at UNDER-conversion.** There was no trigger for the codemod
converting something it should not. That was my gap, and this is what it cost.

## CAUSE 2 — NESTED PATTERNS WERE NEVER REACHED

The checker already names the target form:

> *"retired nested `(Variant binders…)` pattern; a nested variant is `[Variant {:k v}]` (no body),
> or bind the field and match it"*

So the grammar is decided; the codemod simply does not descend into a map pattern's VALUE position.
`recursive_patterns::nested_options_three_levels` and siblings are unmigrated, not undesigned.

## CAUSE 3 — THE REMAINDER, UNDIAGNOSED

`wat_mcp::a_counter_increments_across_turns` (15 type-check errors), `step_match_scrutinee_reduces`,
`probe_arc278_sqlite_interop`, `every_dispatched_verb_is_classified_or_disposed`,
`every_wat_scripts_file_loads_on_the_current_runtime`. **Diagnose before fixing** — do not assume
they share a root with 1 or 2. If any is unrelated to this stone, say so and report it.

## STOP TRIGGERS

- **STOP-1 — ⛔ NEW: an over-conversion.** Any `.wat` site the codemod rewrote that was NOT a match
  arm. `[~@ {` is one known signature (3 sites) — **it is the signature I guessed, not a census.**
  Derive the population from the failures and from the codemod's own diff, not from that pattern.
- **STOP-2 — the loosened recogniser is re-tightened by requiring a `match` parent.** That would
  re-break the spliced `~@serve-op-arms` case the last reland fixed. If you cannot discriminate a
  spliced ARM from a spliced CALL structurally, STOP and report — a marker or an explicit site list
  is a builder decision, not a strike decision.
- **STOP-3 — hand-editing a `.wat` form.** R21, unchanged. Strings in `.rs` remain the one exception.
- **STOP-4 — cause 3 is "fixed" by assuming it shares cause 1 or 2's root.** Diagnose each.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | floor | `0 failed`, read from `^ +Summary` |
| 2 | the 4 probe rows | still PASS, `#[ignore]` 0 |
| 3 | ⛔ over-conversions reverted | `->>`/`->` macro bodies restored; the threading cluster green |
| 4 | ⛔ the spliced-arm case still works | the `defservice` serve-op arms the LAST reland fixed must NOT regress — name the test that proves it |
| 5 | nested patterns migrated | `[Variant {:k v}]` (no body), per the checker's own message |
| 6 | cause 3 diagnosed per test | each named with its root; any unrelated one reported as a separate finding |
| 7 | clippy | 0 |
