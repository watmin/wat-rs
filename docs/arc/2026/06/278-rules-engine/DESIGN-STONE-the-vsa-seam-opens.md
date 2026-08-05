# DESIGN STONE + BRIEF — the VSA seam opens: holon verbs reach the `where` surface

**Ruled 2026-08-05 by the builder.** The form, in his words:

```clojure
(wat.rete.holon/cosine ?a ?b :undefined 0.0)   ;; force the user to choose what unknown means to them
```

> *"the non-similarity branches are :unknown — they shouldn't be reached."*

Marker spelling ruled the same turn: **`:undefined`**, the existing one. Not a second spelling
(arc 179's `()` lesson — a second spelling of one marker is a second door around every wall built on
the first).

Anchor `/home/watmin/work/holon/wat-rs/`; verify with `pwd`. Tree clean at HEAD `7c0753ee`.
Floor **`4352 / 4352 / 0 / 262`**, clippy clean, `check-where-shapes.sh` → `9 pair(s), 98 rows`.

---

# PART I — THE DESIGN

## Why this is the arc's point, not a footnote

R4 designed the engine with a **matcher seam**: *"swap RETE's exact test for coincidence — similarity
over a floor, so rules fire on resemblance, not equality."* The lineage is Clara@Shield → the eBPF
rule trees → this, and R25's chaos engine is rules over streaming anomaly scores. **That seam has
never been opened.** Today the state is:

| layer | state |
|---|---|
| the four VSA verbs classified pure ∧ det ∧ **total** | ✅ `purity.rs`, builder-ruled 2026-08-01 |
| `RETE_MODULES` admits `:wat::rete::holon::` | ✅ `vocabulary.rs:878` |
| `RETE_OPS` holon rows | ❌ **zero** |

A declared namespace with nothing behind it. And it is currently invisible: the third conjunct is
unarmed, so `:wat::holon::cosine` in a `where` passes on pure ∧ det alone. **Arming would silently
delete the seam.** This stone is what makes arming safe.

## ★ The measurement/predicate split does the work — it is already ruled

The builder's 2026-08-02 ruling (`THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT`) partitions these
four, and the partition decides the class of every row:

- A **measurement** returns a scalar and may not absorb its own undefined case — so `cosine`/`dot`
  return outcome enums (the cosine outcome wall, #62), and at the rete surface they carry a
  **fallback**.
- A **predicate** answers the question actually asked, exactly — so `coincident?`/`presence?` already
  return a plain `bool` and **need no fallback**. `coincident?` says so in its own body:
  *"an undefined comparison is not below the floor, so the honest answer to the question actually
  asked ('are these the same point?') is `false`, by documented total contract — never a raise."*

**Do not add a fallback to the predicates.** They are already total, by ruling, on the disk.

## The four rows

| rete_name | core_name | class | shape |
|---|---|---|---|
| `:wat::rete::holon::cosine` | `:wat::holon::cosine` | **`Fallback`** | `[Holon, Holon, Keyword, F64] -> F64` |
| `:wat::rete::holon::dot` | `:wat::holon::dot` | **`Fallback`** | `[Holon, Holon, Keyword, F64] -> F64` |
| `:wat::rete::holon::coincident?` | `:wat::holon::coincident?` | **`Redispatch`** | no scheme — bespoke inference |
| `:wat::rete::holon::presence?` | `:wat::holon::presence?` | **`Alias`** | `[Holon, Holon] -> Bool` |

All four: `meta: OpMeta { pure: true, deterministic: true, total: true }` — transcribed, not decided;
they carry exactly what `purity.rs` already says.

## ★★ THE NEW MECHANISM — a THIRD failure mode for the `Fallback` arm

`dispatch_rete_op`'s `OpClass::Fallback` arm (`runtime.rs:8251`) today faces two, and this is a third
that is shaped like neither:

| family | how the core op fails | what the arm does |
|---|---|---|
| i64 | **raises** `IntegerOverflow` / `DivisionByZero` | catch the `Err` |
| f64 | **returns** a non-finite IEEE value | inspect the `Ok` scalar (`!is_finite()`) |
| **holon** | **returns an outcome ENUM** | inspect the `Ok` **variant** — and **unwrap the happy variant's payload** |

The first two hand back the value unchanged. **This one projects a field out of the wrapper**:
`CosineOutcome::Similarity { similarity }` must become an `f64` at the rete surface. That is not a
wider match arm; it is a different operation, and it is the thing a rider gets wrong by copying
either existing path.

**⚠ THE TWO ENUMS DO NOT SHARE NAMES. Read them, do not pattern-match by memory:**

- `:wat::holon::CosineOutcome` — happy variant **`Similarity`**, field **`similarity`**;
  plus `Degenerate [side <- DegenerateSide]` and `DimensionMismatch [expected <- i64, got <- i64]`.
- `:wat::holon::DotOutcome` — happy variant **`Computed`**, field **`product`**;
  plus `DimensionMismatch [...]`. **No `Degenerate`** — `dot` has no magnitude normalisation, so a
  zero vector is an ordinary input with an ordinary answer.

A row that unwraps `Similarity` from a `DotOutcome` will not compile; one that assumes both have a
`Degenerate` will write an unreachable arm. **Ground both enums in `src/types.rs` before writing the
match.**

## ★ `ParamType` needs a `Holon` variant — measured, not assumed

`ParamType` (`vocabulary.rs:97`) can spell `I64 Bool Keyword String F64 Var PersistentVectorOf
OptionOf`. It cannot spell a holon. The type is `TypeExpr::Path(":wat::holon::HolonAST")` — the
schemes build it with a local closure at `check.rs:14974` (`let holon_ty = || …`), which is why
`grep 'fn holon_ty'` finds nothing; it is not a function.

Add `ParamType::Holon`, converting to that `Path`. Exactly the shape round 1a used when it added
`String` and `F64` for the same reason.

## ★ THE DELIBERATE NARROWING — say it out loud, do not let it be discovered

Core's `cosine`/`dot`/`coincident?` are **polymorphic over `HolonAST | Vector`** in either position
(arc 052/061) — which is why three of the four have no `TypeScheme` at all. A `Fallback` row DOES
register a scheme from its `params`, so declaring `[Holon, Holon, …]` **narrows the rete spelling to
HolonAST-only.**

**That is correct and intended**, by the standing per-type ruling — *"the rete surface is per-type,
period"* — the same rule that made `i64::>` and `f64::>` separate rows rather than one generic. A
`Vector`-typed sibling row is a legitimate follow-up **question**, not an omission, and not this
stone. Record it; do not mint it.

`coincident?` is `Redispatch` precisely because it keeps its polymorphism — that class exists for
"an ordinary fn whose type cannot be stated as a rank-1 scheme" and the checker re-dispatches to
core's own inference.

## The expression this makes possible — and today's stone was its prerequisite

```clojure
(where (:wat::rete::f64::> (:wat::rete::holon::cosine ?a ?b :undefined 0.0) 0.9))
```

Without `:wat::rete::f64::>` — minted this session — there is nothing to compare the unwrapped
scalar against. The f64 surface being a two-row stub is exactly what blocked this.

## ⛔ A STALE COMMENT TO FIX IN THIS STONE

`src/rete/purity.rs` (the VSA-seam block, ~`:450`) writes the motivating expression as:

```clojure
(:wat::core::f64::> (:wat::holon::cosine ?a ?b) 0.9)
```

**That cannot type-check.** It predates the cosine outcome wall; `cosine` returns `CosineOutcome`,
not an f64. Worse, it is the *precise shape the wall was built to prevent* — a guarded `0.0` sailing
through `(f64::> … 0.9)` as a confident no-match, when genuine unrelatedness reads `-0.0086`.

Replace it with the fallback form above. A comment naming a shape the code no longer has is a lie the
next reader inherits.

---

# PART II — THE STRIKE

## Read in order

1. This document, whole.
2. `src/types.rs` — `CosineOutcome` (~`:2206`) and `DotOutcome` (~`:2262`). **Both**, for their
   variant and field names.
3. `src/runtime.rs` — `dispatch_rete_op`, the whole fn. Its `Fallback` arm's comment is careful about
   exhaustiveness; match that voice when you add the third mode.
4. `src/rete/vocabulary.rs` — the f64 `Fallback` quartet (minted this session) is your row exemplar;
   `ParamType` (~`:97`) is where the new variant goes; `RETE_MODULES` (~`:873`) already admits the
   namespace — **do not edit it** (STOP-3).
5. `src/rete/purity.rs` — the VSA-seam block for the stale comment, and to confirm (not change) the
   four verbs' classification.
6. `src/check.rs:14974` — how a holon type is spelled.

## Order of work

**A.** `ParamType::Holon` + its `to_type_expr` arm.
**B.** The `Fallback` arm's third mode (unwrap-or-fallback on the outcome enums).
**C.** The four rows.
**D.** The stale comment.

**A and B must precede C** — the rows cannot be spelled without the variant, and their `total: true`
is unearned until the arm faces the outcome. Same reasoning as the f64 stone's STOP-1.

## ⛔ STOPs

- **⛔ STOP-1 — do NOT give `coincident?` or `presence?` a fallback.** They are predicates and are
  already total by the builder's ruling. Adding one would re-open the hole the ruling closed.
- **⛔ STOP-2 — do NOT touch core.** `:wat::holon::*` keeps returning its outcome enums; the *rete*
  row is where the fallback lives. Core's honesty is the thing being surfaced, not changed.
- **⛔ STOP-3 — do NOT edit `RETE_MODULES`.** `:wat::rete::holon::` is already there.
- **⛔ STOP-4 — mint exactly these four.** The other 101 `:wat::holon::` verbs are deliberately
  unclassified (`purity.rs:150` names the three groups and why). Not this stone.
- **⛔ STOP-5 — do not mint a `Vector`-typed sibling.** The narrowing is intended; record it in your
  report as an open question.
- **⛔** No `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, run the suite SOLO

```
cargo build --release
cargo nextest run --release          # no other cargo process alive
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Read the **Summary line** — never a piped exit code.

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | row count | **57** (53 + 4) |
| 2 | ★★ **the seam opens** | `(:wat::rete::f64::> (:wat::rete::holon::cosine a b :undefined 0.0) 0.9)` type-checks AND runs on two similar holons → `true` |
| 3 | ★★ **the fallback FIRES on a degenerate operand** | build a zero-magnitude vector — `(vector-blend v v 1.0 -1.0)`, **proven reachable on disk** (`probe-zero-magnitude-reachable.wat`) — and the row returns the caller's value, not `0.0` |
| 4 | ★★ **non-vacuity** | same degenerate expression, fallback `-1.0` vs `7.0` → `-1.0` then `7.0`. Rows 2–3 pass on a constant; only this proves the caller's value is returned |
| 5 | ★ **the happy payload is UNWRAPPED, not the enum** | the row's value is an `f64` usable directly by `f64::>` — if it returned the enum, row 2 would not type-check |
| 6 | ★ `dot` unwraps `Computed.product` | `(:wat::rete::holon::dot a b :undefined 0.0)` returns the product as f64 |
| 7 | ★ the predicates need no marker | `(:wat::rete::holon::presence? a b)` and `(… coincident? a b)` are **2-arity**, return bool, no `:undefined` |
| 8 | ★ the narrowing is real and located | passing a `Vector` where the row declares `Holon` is a **located type error at `--check`**, exit 1 |
| 9 | ★ i64/f64 fallbacks unregressed | `(:wat::rete::i64::/ 1 0 :undefined -1)` → `-1`; `(:wat::rete::f64::/ 0.0 0.0 :undefined -1.0)` → `-1.0` |
| 10 | ★ the stale comment is gone | `purity.rs` no longer contains `(:wat::core::f64::> (:wat::holon::cosine` |
| 11 | ★ floor | `4352 / 4352 / 0 / 262` or higher; **nothing lost** |
| 12 | ★ gate | `9 pair(s), 98 rows` |
| 13 | clippy | clean |

Rows 2, 3, 4, 5, 8, 9, 11, 12 re-run by the orchestrator by hand.

**Runtime prediction: 60–90 minutes.** Three mechanisms (a `ParamType` variant, a runtime mode, four
rows) plus a probe that must construct holons. Time-box 180.

**Trap doors:**

1. **Copying the f64 mode.** It inspects a scalar. This inspects a variant *and unwraps a field*.
2. **Assuming both enums match.** `Similarity`/`similarity` vs `Computed`/`product`; `Degenerate`
   exists only on `CosineOutcome`.
3. **Giving the predicates a fallback.** STOP-1 — they are already total by ruling.
4. **A probe that never reaches the degenerate case.** Row 3 is the whole point; if you cannot build
   a zero-magnitude holon, STOP and report rather than asserting the arm works.
5. **Skipping row 4.** Rows 2–3 pass if the arm returns a constant.

## Scratch

Scratch `.wat` goes in `wat-scripts/scratch-pad/` — never a tmp dir; that directory is parsed and
type-checked on every build, which is the point. Write a real program with a `:user::main`; a probe
without one fails before resolving anything and proves nothing. If row 8 needs a negative fixture it
goes in `tests/` with a rust test asserting the failure — a `.wat.bad` is a contract with a test on
the other end.
