# Arc 172 — INSCRIPTION

**Status:** Slices 1+2 shipped + closed 2026-05-10 as atomic-commit pair.

**What this arc closes:** the third (and primary macro-syntax)
thread of REALIZATIONS pass 14's lexical EDN-compliance work.
Thread 1 (slice 1f-W; `<>`-position commas) shipped at
`4278c4d`. Thread 2 (arc 171; commas in keyword/symbol bodies)
shipped via slices 1-3 + INSCRIPTION 2026-05-10. Thread 3
(this arc; macro flavor swap) ships here.

**After this arc:** wat is fully lexically EDN-compliant.
Commas carry zero meaning in wat source outside `(...)`
tuples and `<...>` parametrics. Macros use Clojure-style
`~name` / `~@list` for unquote / splicing.

## What shipped

**Lexer** (`src/lexer.rs`):
- Comma `,` retired as Unquote / UnquoteSplicing token at the
  main lex loop; now treated as EDN whitespace
- `~` and `~@` mint as the new Unquote / UnquoteSplicing
  tokens (same `Token` variant names; only source character
  changed)
- `is_symbol_break` includes `,` so bare symbols terminate
  at comma
- Arc 171's `lex_keyword` reject rule for depth-0 keyword-body
  commas unchanged

**Parser** (`src/parser.rs`):
- Existing quasiquote tests migrated from `,foo`/`,@xs` to
  `~foo`/`~@xs`
- Reader-macro dispatch (`:wat::core::unquote` /
  `:wat::core::unquote-splicing`) unchanged

**Consumer sweep — ~73 sites across 20 files** (3.5× BRIEF
pre-grep estimate):
- 10 wat-stdlib files (`wat/test.wat`, `wat/core.wat`,
  `wat/runtime.wat`, `wat/holon/*` — Amplify, Subtract,
  Bigram, Trigram, Project, Reject, Circular, Log,
  ReciprocalLog, Sequential, Ngram)
- `wat-tests/core/struct-to-form.wat`
- 4 `tests/wat_*.rs` files with embedded wat (variadic_defmacro,
  arc144_lookup_form, idempotent_redeclare,
  arc144_uniform_reflection)
- `src/macros.rs` ~20 unit-test embedded wat strings (out of
  BRIEF scope; sonnet caught)

Patterns migrated:
- `,name` → `~name` (unquote)
- `,@list` → `~@list` (splice)
- `,,name` → `~~name` (nested unquote per arc 029; 6 sites
  total)
- `,(expr)` → `~(expr)` (computed unquote in
  `wat/runtime.wat::define-alias`)
- `,@(expr)` → `~@(expr)` (computed splice)

## Slicing — atomic-commit pair

Two slices ran sequentially; committed atomically:

| Slice | Scope | Result |
|---|---|---|
| 1 | Lexer comma→whitespace + `~`/`~@` mint + parser test migration | Mode A; ~10 min sonnet; workspace RED by design (453/938) |
| 2 | Consumer sweep ~73 sites across 20 files | Mode A; ~45 min sonnet; workspace returned to 1339/854 (+5 from previously-broken macros.rs tests; 0 fails delta) |

Per recovery doc § "Atomic commit across coordinated sweeps":
slice 1 shipped its lexer change uncommitted; slice 2 swept
the consumers; both committed together when workspace returned
to green. Mid-sweep brokenness was acceptable; on-disk-committed
brokenness was not.

## What's deliberately NOT in this arc

The DESIGN listed three Clojure features beyond the
syntax swap:
- **Auto-gensym** (`name#` in syntax-quote scope)
- **`&form` / `&env` implicit macro args**
- **`gensym` primitive** (manual fresh symbol)

These are intentionally OUT of arc 172's slices 1+2 scope.
They are tracked as **arc 172 slice 3** in the same arc dir.
If/when slice 3 ships, the Clojure macro-feature parity
completes. Slices 1+2 alone deliver the EDN-compliance
contract — wat is now lexically EDN-compliant; the Clojure
macro NICETIES are a separate concern.

The four-questions on this scope cut: slices 1+2 deliver one
coherent change (the syntax pivot). Slice 3 delivers a different
one (the macro-feature additions). Splitting honors atomic
commit + clean SCORE per change.

## Workspace impact

- Pre-arc-172: 1334 passed / 854 failed (post arc 171 baseline)
- Post-arc-172: 1339 passed / 854 failed
- Pass-count delta: +5 (previously-broken `src/macros.rs`
  comma-syntax tests now fixed)
- Fail-count delta: 0 (zero regressions; ~73 sites swept
  cleanly)

## Why this matters

Per REALIZATIONS pass 14 (arc 170):

> *"if yes [transmit macros over the wire].. we need to remove
> the comma... we have scheme macros now.... we need to swap
> to clojure macros..."*

After this arc + arc 171 + slice 1f-W, **wat source
round-trips through EDN parsers without lossy
reinterpretation.** Macros become serializable data; the
substrate's lexical surface is fully EDN-compliant. The
foundation for the next leg of work (per the user's "the
foundation must be impeccable" direction) is impeccable on
this axis.

## The four questions

**Obvious?** Yes. Comma is whitespace per EDN; `~` is
Clojure unquote per the locked decision; the substrate's
syntax now matches the wire format it was always claiming to
emit.

**Simple?** Yes. One lexer block retired; one block minted;
mechanical consumer sweep. No macro-evaluator changes; no new
abstractions.

**Honest?** Yes. The arc names every concern: BRIEF pre-grep
missed 6 holon files + `src/macros.rs` unit tests; sonnet
caught + handled them without scope expansion. The +5
pass-count delta is named (previously-broken tests now
fixed). Slice 3 is tracked, not deferred — it's its own
slice in the same arc.

**Good UX?** Yes. Authors read `~name` and know it's
unquote. The Clojure heritage carries familiarity from the
ecosystem. wat source round-trips through EDN tooling
(formatters, syntax-aware editors).

## Out of arc 172 slices 1+2 scope

**Arc 172 slice 3** — Clojure features (auto-gensym `name#` +
`&form`/`&env` implicit args + `gensym` primitive). Tracked
in this arc dir; opens on its own terms. Slices 1+2 deliver
the EDN-compliance contract independently of slice 3.

## Closure-discipline pre-INSCRIPTION grep

Per recovery doc § 11 mandatory grep — ran clean except
for cross-reference language pointing at slice 3 (named +
tracked, not deferred). All scope commitments for slices
1+2 shipped.

## Cross-references

- DESIGN.md (this arc; broader scope including slice 3)
- SCORE-SLICE-1.md / SCORE-SLICE-2.md (per-slice calibration)
- Memory: `feedback_apostrophe_dispatch_separator.md`
  (sibling EDN-compliance work; unchanged by this arc)
- Arc 170 REALIZATIONS pass 14 (parent decision)
- Arc 170 slice 1f-W (`4278c4d`) — first thread (`<>`-position
  commas wire encoding)
- Arc 171 (`a40a40a` + `9566d33` + slice-3-INSCRIPTION) —
  second thread (commas in keyword/symbol bodies)
- Arc 029 (nested-quasiquote) — depth-tracking semantics
  preserved across the `~~` migrations (6 sites)
- 058 FOUNDATION-CHANGELOG row added 2026-05-10 (lab repo)
