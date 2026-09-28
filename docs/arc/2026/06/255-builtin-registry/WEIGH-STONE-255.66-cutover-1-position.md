# WEIGH — STONE 255.66: cutover 1/7, position decides — ACCEPTED

**Executor: grok via pulsare (its last strike before its credits ran out), commit `ae97d8092`.** Weighed by the
orchestrator on 2026-09-27.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6200 passed / 24 skipped** (6193 + 7 rows) |
| `wat --check wat-scripts/scratch-pad/probe-arc255-66-the-534.wat` | rc 0: `conj`/`nth`/`foldl`/`mapv`/`select` over `(wat.type/Vector :- [wat.type/AST])` check against today's stdlib |
| the helper | read: `constructor_head_key` (`src/types.rs:141`) denotes the head and, for the seven constructor keys, dispatches on the old key; any other head is unchanged |
| census / delta / clippy | grok's: 0 flips; NEW 2 / RECOVERY 0 same files; clippy rc 0. Not re-run |

## The reading

- **255.64's 534 errors were the recurring class, as suspected:** `StreamContainer::of_type` compared a stored head to
  the string `"wat::core::Vector"`. `format_type` had already denoted the head, so the message showed a head the match
  had not accepted. Same compare in `infer_select_prime`. Cured at one door (`parametric_heads_unify` /
  `constructor_head_key`) across ~20 functions.
- **The heresy ledger fell 195 → 149.** Twelve spelling-compare rows are gone and eight functions shrank. The first floor's
  two reds were that ledger's own accounting and a moved rune, kept and fixed, not re-run away.
- Position holds: `(wat.type/Vector :- [T])` is the type in a parameter and the empty vector in value position; with
  items it equals `[1 2 3]`, in both spellings, for all seven constructors (`List` takes its type as an annotation).
  P1 is in the macro-purity walk.
- **The door carries a hand list** (the seven constructor keys in `constructor_head_key`). Under T-door that is the
  temporary door, and **stone 4 (C1) deletes it**. Recorded so it cannot outlive the stone that owns its removal.
- The denoted empty `(Tuple :- [A B])` falls through as a type form (`src/check.rs:3408`), so the two
  `keyword/to-type-form` goldens stay rc 0. The old spelling's answer (expected 2, got 0) is unchanged.

## Next

Stone 2 of 7 (codemod, types only) needs the builder's question 1: which of `u8`, `bigint`, `rational`, `char`, `nil`,
`Value`, `PersistentVector`, `PersistentMap`, `Bytes` are hard primitives in `wat.type/`. Executor from here: Sonnet
subagents (grok's credits are out for a few days).
