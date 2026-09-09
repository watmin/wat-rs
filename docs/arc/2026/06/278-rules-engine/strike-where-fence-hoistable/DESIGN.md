# DESIGN — the `where` fence's interior is unvalidated, and the gate is the deliverable

**Status:** drawn 2026-09-09 from a bug report raised on the `compute` host (`sns-sqs`).
⛔ **Not a vigilia row** — the vigilia read this file and did not find this. Why is below.

## The hole

`src/rete/validate/mod.rs:282`, in `validate_when_entry`:

```rust
// Design call 3 — a `where` fence's outer shape is already confirmed by the
// classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
ReteClauseShape::Where(_) => {}
```

The fence's **outer** shape is checked; its **interior** is never looked at. So a `where` carrying a
constraint that could legally sit **inline in the condition that binds its vars** — an alpha test
rather than a beta one — is accepted silently. `matcher.rs:304`'s `cond_has_deferred_constraint` is
the existing alpha/beta discriminator and is the inverse of the predicate this needs.

## ⛔ Why the vigilia missed it, because it bears on how much else is missing

Twelve rows cite `validate/`; **eight wards read it** (`purgare`, `conferre`, `sequi`, `perspicere`,
`probare`, `excusare`, `intueri`, `solvere`). **Zero reports mention `Design call 3` or `Where(_)`.**

**The ward whose subject this is was mustered out.** `vigilia-2026-09-07-rete/README.md:46`:
*"`peragrare` | a load-bearing instrument AND its corpus | **NO here** — fires on target 3."* The
validator **is** a load-bearing instrument and its `Where` branch **is** an unvisited cell — exactly
what `peragrare` names. It was aimed at the grid instead.

And the eight that did read it were each aimed elsewhere: the arm is not dead (`purgare` — it is a
deliberate no-op), contradicts no spec (`conferre` — nothing claims it validates), and carries a real
justification (`probare` — the comment **is** substance). ⭐ **It looks exactly like what it is: a
considered decision that the corpus has since outgrown.** No inward lens is aimed at *"this was right
when written."*

## Census — RE-DERIVED here; do not reuse the compute figures

The report's census (231 in `wat-scripts/fixes/`, 92 elsewhere, 0 in `wat/`) was measured on a branch
**586 commits divergent** that never saw this branch's work. On `grok-rete`:

| tree | files | `:wat::rete::where` hits |
|---|---|---|
| `wat-scripts/` | 71 | 277 |
| `tests/` | 36 | 78 |
| `.rs` string literals | 24 | — hand work |
| **`wat/`** | **2** | **7** |

⚠ **The `wat/` hits look like a bootstrap hazard and are NOT one — I checked.** All seven are string
comparisons against the head name (`compile.wat:92,126,370,376,917`, `oracle/pass.wat:542,608`) —
the compiler and oracle **recognising** the form, never using it in a rule. **No bootstrap dance**,
same conclusion as the report, reached independently.

⛔ **A hit is not a refusable site.** Refusable means *the constraint could legally move into a
condition that binds its vars*. Deriving that population is the strike's first act, not an
assumption.

## What this delivers — and the gate is the point

An upcoming merge into `main` will conflict in this file (`validate.rs` on one side,
`validate/{mod,error,typing}.rs` on this one). **A reference branch is one where a bad resolution
REDDENS.** So:

1. **The check**, filling `:282`.
2. **A gate that fails if the arm goes empty again** — the deliverable that survives a merge.
3. **A refusal that teaches**: name the variable, name the condition that binds it, and **print the
   rewrite**. House style — `UnknownField` carries `available_fields`.

## The one contract decision, pinned

⛔ **THE PREDICATE IS "∃ A CONDITION THIS CONSTRAINT CAN LEGALLY MOVE INTO", NOT "∃ A CONDITION THAT
BINDS THESE VARS".** `validate/mod.rs:106-110` pre-wrote this hazard for a different decision:

> *"needs binder analysis over `:when` (an `Or` arm binds conditionally, `exists` binds nothing
> outward, `accumulate` binds its result var) — and **under-collecting that set would reject LEGAL
> rules, which is the one failure a wall must not have.**"*

`Or` / `exists` / `accumulate` are exactly where the two predicates diverge. **A fixture proving a
legal rule is still accepted must exist BEFORE the wall goes in.**

## ⚠ TWO arms, not one — and they are justified differently

| site | justification |
|---|---|
| `validate/mod.rs:282` — the fence | *"its interior expr is out of scope"* — **scope.** This strike's target |
| `validate/mod.rs:456` — within-condition | *"the **stone-6 STOP arm** (always `None` at fire time)"* — **a runtime fact** |

**A cure pattern-matching `ReteClauseShape::Where(_) => {}` hits both.** `:456` is out of scope here;
if it should change, that is a separate argument.

## Out of scope = rejected

- **`:456`.** See above.
- **Migrating the corpus.** A codemod over refusable sites is a `wat-fix` strike of its own; this one
  makes them *detectable*.
- **The 24 `.rs` string-literal sites.** Hand work, and not this strike's.
