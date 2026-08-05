# DESIGN STONE — a `where` body is CODE, and the expander must treat it as code

> **Status: RULED 2026-08-05 by the builder — *"we should just expand where bodies at compile
> time."*** Unblocks `cond` in a `where`, and with it every future macro-backed rete op.
> Prerequisite: `BRIEF-rete-cond-is-its-own-macro.md` (the expansion must EMIT rete spellings, or
> this stone just delivers core-spelled forms into an armed fence).

## The defect

A `where` body is **never macro-expanded**. Grounded three ways, each by my own read:

| site | what it does |
|---|---|
| `wat/rete.wat:2315` | `defrule`'s template quotes the conditions verbatim — `(:wat::core::quote ~when-vec)` |
| `src/macros/expand.rs:441` | the expander returns early on `quote`/`quasiquote`/`literal` — *"carry DATA, not code"* |
| `src/rete/matcher.rs:1237` | `eval_test_core` calls `runtime::eval_inner` on that raw AST — never consults the macro registry |

So a macro inside a `where` is invisible at expand time and unknown at fire time. Proven by run,
both spellings: `probe-cond-in-where-baseline.wat` (core) and `probe-cond-rete-where.wat` (rete)
both raise `#wat.runtime/UnknownFunction {:message "unknown function: :wat::core::cond"}` at fire.

**The consequence is bigger than `cond`: no macro can ever join the `where` vocabulary while this
holds.** The form mirrors work only because `if`/`let`/`match`/`fn` are genuine runtime special
forms. `cond` is the first that is not, and it will not be the last.

**And the control proves the target is reachable:** `probe-rete-if-in-where.wat` —
`(:wat::rete::where (:wat::rete::core::if ?a true false))` inside a real `defrule` — prints
**`hits=1`**. Expansion is the *only* missing link, not the first of several.

## Two routes are already closed — do not re-derive them

1. **`defrule` expands its own `:when`.** Blocked by a deliberate wall: `macroexpand` /
   `macroexpand-1` are **excluded from the macro-eval `is_pure_total` allow-list**, and calling one
   from a macro body raises `MacroErrorKind::RefusedInMacro`. The guard has a standing witness test —
   `macros/tests.rs:1365 macroexpand_in_computed_unquote_refused_with_refused_in_macro`, whose own
   doc says *"the gate bites its author."* This is a decision, not a gap.
2. **Expand at `:wat::rete::compile`.** That is a *runtime* call; macros are consumed at freeze and
   gone. (The builder's "compile time" is expand time — in wat they are the same phase of freeze.)

## ★ THE MECHANISM — it already exists, and it is one classification

`src/resolve/boundary.rs` is **the single source of truth for "which of this head's arguments are
code and which are data."** Its own header records why it exists: `walk`, `normalize` and
`expand_form` each carried a hand-rolled `if`-chain, the chains **drifted** (arc 251.1 ward), and
the classification was decomplected into one enum.

The precedent is exact. `Boundary::MatchesSubject` — *"`:wat::form::matches?` — only the subject
(`items[1]`) is code; the pattern (`items[2..]`) is DSL data"* — and `expand_form:458` consults it
to expand **one child** and leave the rest untouched. A `where` body is the same shape.

**⇒ Add a `Boundary` variant. `walk` and `normalize` match it EXHAUSTIVELY, so adding one is a
compile error in both until handled** — the substrate hands back the worklist (R65
`SCVTVM IDEM INDEX`), which is the reason to put it here rather than in a fourth `if`-chain.

### ★★ THE HOOK IS `make-rule`, NOT `defrule` — measured, and this is the load-bearing choice

A census of rule producers (`grep 'make-rule' --include=*.wat`, non-doc, non-comment):

| producer | site |
|---|---|
| `defrule`'s template | `wat/rete.wat:2314` |
| **`sift-rules-defsvc`** — the sift engine's generator | **`wat/query.wat:189`** |
| a hand-built rule literal | `wat-scripts/scratch-pad/probe-rule-lits.wat:33` |
| a **direct** `make-rule` call | `wat-scripts/scratch-pad/probe-sift-body-direct.wat:14,17` |

**Hooking on `:wat::rete::defrule` would silently miss the sift generator and every direct call.**
`:wat::rete::make-rule` is the one door all four funnel through — the same ONE-DOOR reasoning as
#75 and as `RETE_OPS` itself.

The classification to add, stated precisely:

> **`:wat::rete::make-rule`** — `items[1]` (the rule name) is ordinary code. `items[2]` (the quoted
> `:when` vector) is **DATA, EXCEPT the BODY of each `(:wat::rete::where …)` form inside it, which
> is CODE**. `items[3]` (the quoted `:then` vector) is data.

## ⛔⛔ THE HAZARD, AND IT HAS ALREADY BITTEN ONCE

**Expand ONLY the `where` body. Never the surrounding condition patterns.**

A condition vector holds fact patterns — `(:probe::Req (?a <- :a))` — whose heads are
**aggregate-shaped**. Post arc-294 item 9a's construction flip, an aggregate name is a **registered
kwargs companion macro**. Walk the pattern as code and `kwargs-lower` fires on raw DSL clauses as
if they were kv-pairs.

This is not a hypothetical: it is verbatim why `MatchesSubject` exists.
`src/macros/expand.rs:445-455` documents the identical failure for `matches?` patterns —
*"finds an aggregate-shaped pattern head (e.g. `:test::PaperResolved`) that is now a registered
kwargs companion macro, firing `kwargs-lower` on raw DSL clauses as if they were kv-pairs."*

**A `where` body is the only code region inside a condition vector. Everything else stays data.**

## STOPs

- **⛔ STOP-1 — the hook is `make-rule`, not `defrule`.** Four producers; the sugar is one of them.
- **⛔ STOP-2 — expand the `where` BODY only.** Not the condition patterns (the hazard above), not
  the `:then` vector.
- **⛔ STOP-3 — the classification goes in `resolve/boundary.rs`, never a fourth `if`-chain.** The
  module exists because three chains drifted. Adding a fifth encoding of one language fact is the
  defect it was built to kill.
- **⛔ STOP-4 — if `walk` / `normalize`'s exhaustive matches disagree about what the new variant
  means for THEIR pass, STOP and report.** They resolve call heads and rewrite symbol refs; a
  region that is data for one and code for another is a real finding, not a formality.
- **⛔ Do not claim `cond` works in a `where` until a run says so.** The acceptance test is
  `probe-cond-rete-where.wat` going from `UnknownFunction` to a real hit count.
- **⛔** No `_` wildcard arm on an enum scrutinee — `Boundary` is matched exhaustively by design.

## What this unblocks, and what it does not

**Unblocks:** `cond` in a `where`; every future macro-backed rete op; and it removes the
"form mirrors must be runtime special forms" constraint that has silently shaped the vocabulary.

**Does not touch:** the `:then` vector (the RHS is a different question — task #61 already ruled
derived fact fields are copies only); the accumulator fence; and #49a's compiled `where`, whose IR
is specified by the vocabulary and is unaffected by *when* the vocabulary is expanded.

**⚠ Blast surface to weigh, not assume:** 38 `.wat` files contain a `rete::where` form. Expansion
is a behaviour change for every one of them — a `where` body that previously reached `eval_inner`
raw now reaches it expanded. For a body with no macro in it, expansion is the identity; the gate is
`check-where-shapes.sh` (`9 pair(s), 98 rows`, `wat == Clara` on every shape) plus the full floor,
and **a single moved derived fact is the alarm.**

## Order

1. `BRIEF-rete-cond-is-its-own-macro.md` — the expansion must emit rete spellings. **In flight.**
2. This stone.
3. Then #57 may arm the third conjunct — **and not before**, because arming makes the rete spelling
   mandatory, and until (2) lands the mandatory spelling does not fire.
