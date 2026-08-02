# DESIGN-STONE — the rete expression language

> **Status: DESIGNED 2026-08-02, builder-ruled, scope widened mid-design.** Supersedes T2/T3 of
> `DESIGN-STONE-total-the-third-axis.md` (T1 — the axis, unarmed — is LANDED at `f37c54f3`).
> Downstream beneficiary: `DESIGN-STONE-compiled-where.md`.
>
> An earlier draft of this file scoped the work to *arithmetic only*. That was too small; see
> **§ Why the corpus is the wrong instrument for sizing this**.

## The thing being built

A **closed expression language for rete conditions** — the vocabulary legal inside a `where` (and the
accumulator fence). Pure, total, binary where arity is a choice, and closed enough that a compiler for
it is finite.

Builder's framing, which is the design's spine:

> *"think of sql… the ops there are bespoke to sql… in wat everything is edn so our ops LOOK similar
> to regular code… but we can impose exact guardrails on our where clauses to make compilation
> trivial… Clara chose to use regular clojure forms… that means perf takes a hit and there's
> impurities allowed… we can impose a dsl that requires all forms are compilable."*

## ★ THE FORK TO RULE FIRST — the law is namespace-based or property-based

Both have been stated and they build different things:

| | law | consequence |
|---|---|---|
| **A — namespace** | a `where` admits only `:wat::rete::` ops | every op needs a rete name, **even ones already total**. The admitted set is knowable by *reading the head*, so the compiler's dispatch is a closed table. Holon ops need mirrors. |
| **B — property** | a `where` admits only pure ∧ total ops, any namespace | no mirrors; `total: true` classification suffices. But the admitted set is *whatever happens to be classified*, so the compiler still faces an open head-space. |

**Compilability is the whole argument for the closed set, and only A delivers it.** B gives totality and
nothing else — it is today's fence with a third conjunct. Everything below assumes A; if the ruling is
B, most of this stone collapses to "classify things and arm."

## Why — three legs, and only two are measured

| leg | claim | status |
|---|---|---|
| **Totality** | partial ops in a `where` are a live hazard: `first`-on-empty compiles, fires until one empty vector, then aborts the whole fire | **MEASURED** — 39/98 corpus rows refused, 8 verbs (T1) |
| **Coherence** | a partial mint leaves a namespace seam mid-expression with no visible logic to which side a verb falls on | structural |
| **Compilability** | a closed head-space turns `where`-compilation from *"compile a large fraction of wat"* into a finite jump table | structurally true; **payoff UNMEASURED — see the bound** |

### ⚠ The perf leg is bounded, by our own prior stone

`DESIGN-STONE-compiled-where.md`'s Step 0 is a STOP gate written against exactly this temptation:

> *"`filter` is the whole filter-pass loop, not the predicate… a per-TestNode `new_tokens = ts.clone()`
> — at `[50 200]` that is 10,000 Token clones per round… **not the predicate at all**. 'filter is
> 89.5%' ≠ 'the predicate is 89.5%'."*

It cites `[[feedback_measure_the_decomposition_never_read_it]]`. Task #50 tracks that clone separately.
**Totality and coherence justify this build on their own. The size of the speedup is Step 0's to
report, and nobody may say "shockingly faster" before it does.**

Grounded: `compiled_cond.rs` and `compiled_rhs.rs` exist; there is **no** `compiled_where.rs`; `where`
runs `eval_test_core` (`matcher.rs:1125`) from four sites in `kernel.rs`.

### The Clara contrast is R18 one layer down

Clara admits arbitrary Clojure in `:test`. That is *why* it cannot compile the predicate and *why*
impurity leaks into conditions. The scope-reduction is not a smaller engine — it is the weapon, the
same shape as the purity advantage at the negation layer, now at the `where` layer.

## Why the corpus is the wrong instrument for sizing this

T1 measured **8** partial verbs across 98 rows. That is a **floor, not a target**:

> `[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]` — *the corpus is a record of what
> happened to COMPILE, so it is structurally blind to the fence.*

Those rows contain what was expressible under the *old* rules. They cannot say what someone will want
to write once the fence is a real boundary. Sizing this design to them would be designing to
survivorship — and it is the mistake this stone's first draft made.

## ★ Why `:undefined` rather than guards — `if` IS lazy, and it does not help

**Measured this session:** `(if (= 0 0) 42 (:wat::core::i64::/ 1 0))` prints `42`. `if` evaluates only
the taken branch, so a hand-written guard is genuinely safe *at runtime*.

**But the fence is a STATIC WALK.** `classify_expr` visits every sub-expression including the untaken
branch, sees `i64::/`, and refuses. Accepting a guard would require the checker to *prove the guard
sufficient* — theorem-proving, inside a checker deliberately kept magic-free.

**`:undefined` is what makes totality structural instead of provable.** That is the justification; it is
not a workaround for missing laziness.

## What "total" MEANS — builder-ruled, stricter than IEEE

> *"NaN /is/ an undefined here… force the users to deal with what happens when they get 'not-a-number'
> or they hit a positive or negative infinity — all of these are 'undefined' in my mind."*

**Total = produces an ORDINARY VALUE on every input. Not "never raises."** IEEE achieves totality by
sentinel; a sentinel is the substrate's own name for undefined, and NaN is worse than a raise because
*every comparison against it is false* — the rule silently does not fire.

**Forces a re-audit of T1's `total: true` entries** (bounded to those, not the 110-verb list). One
question each, answered by reading the implementation: *can this produce NaN or ±Inf?*

```
f64::+  f64::-  f64::*            <- overflow to ±Inf        => NOT total
bigint::to-f64  rational::to-f64  <- unbounded source → +Inf => GROUND IT, likely NOT total
i64::to-f64                       <- i64 max ≪ f64 max        => keep
bigint::+ -   rational::+ -       <- arbitrary precision      => keep
rational/numerator /denominator   <- accessors                => keep
```

**The closure property this buys:** if every *producer* of NaN/±Inf requires `:undefined`, NaN cannot
arise inside a `where`, so comparisons are safe by construction.

**⚠ The hole, stated not papered:** a *fact field* may already hold NaN. The fence governs expressions,
not data. An ingested NaN reaches a comparison and silently answers false. This design does not close
that.

## The vocabulary

A rule *condition* needs a specific slice of wat — no IO, no spawn, no services, no mutation.

| family | shape |
|---|---|
| arithmetic `i64` | `+ - * / mod rem quot` — **`:undefined` required** |
| arithmetic `f64` | `+ - * /` — **`:undefined` required** (per the NaN ruling) |
| comparisons | `< > <= >= = not=`, both types — plain; safe by the closure property |
| boolean | `and or not` — plain |
| field accessors | declaration-derived, already total — plain |
| collection reads | `length`, `contains?`, `get` plain; **`first`/`nth` take `:undefined`** |
| string reads | `length`, `concat`, `starts-with?`, `ends-with?`, `to-lowercase`, `trim` plain; **`subs` takes `:undefined`** |
| **holon similarity** | `cosine`, `dot`, `coincident?`, `presence?` — the VSA seam; see below |
| bounded folds | `foldl` — already handled conditionally via fn-arg recursion |

Call it 40–60 names, **most already total and needing only a rete presence** under law A.

### Forms are NOT ops — no `:wat::rete::if`

`if` / `cond` / `let` / `match` / `fn` are **forms**, not functions: no domain, so they cannot be
partial, and `classify_expr` already special-cases them structurally (its `cond` arm: *"a clause is NOT
a call; every element (test AND body forms) is an expression that must satisfy the axis"*). You
namespace functions, not syntax — SQL has `CASE WHEN` and `ABS()`, and only one is a function.

### The holon seam — a CLASSIFICATION, not a mint

Four verbs (`cosine`, `dot`, `coincident?`, `presence?`) were ruled pure **2026-08-01** precisely to
open R4's designed VSA-matched-LHS seam; `purity.rs:945` records that leaving them unclassified had
*"welded shut R4's designed VSA seam."* The canonical form is written into `purity.rs:370`:

```clojure
(:wat::core::f64::> (:wat::holon::cosine ?a ?b) 0.9)
```

**All four are `total: false` today** (T1 default-deny; no corpus row uses them). **Arming without them
re-welds the seam shut, one day after it was opened.**

**And cosine CANNOT produce NaN** — grounded in the sibling repo,
`holon-rs/src/kernel/similarity.rs`:

```rust
let denom = (na * nb).sqrt();
if denom < 1e-10 { 0.0 } else { dot / denom }
```

The degenerate case is already guarded. Designing a `:undefined` for it would be
`[[feedback_an_unreachable_arm_accumulates_lies]]`. So the four are **likely already total** — this is
four grounded classifications, not four fallback-carrying mints. (`dot` still needs its own check: can
a large-dim f64 vector reach ±Inf?)

**⚠ OPEN, and it follows from the NaN ruling:** that guard returns `0.0`, which in cosine's semantics
means *"orthogonal, unrelated"* — a plausible-looking answer for an undefined comparison, and arguably
worse than NaN because NaN propagates visibly while `0.0` sails through `(f64::> … 0.9)` as a confident
*no match*. Should `rete::holon::cosine` require `:undefined` for the degenerate case, so the rule
author states what comparing against a zero-magnitude vector means? **Builder's call.**

## The call shape — RULED, and probe-verified

```clojure
(:wat::rete::i64::+ a b :undefined -1)
(:wat::rete::f64::/ x y :undefined 0.0)
(:wat::rete::first  xs  :undefined 0)
```

Positional operands in core's order; **`:undefined` is the only kwarg**. Killed full-kwargs with one
form — *"`(+ :1 0 :2 2 :3 massive-int)` …. wtf is a kwarg for `+`?"* The principle: positional
confusion is a defect only where there is **no established order convention**. Division's order is
universally known; naming it buys nothing on a form written constantly.

**Probe-verified, `--check` AND a run, both directions:** mixed positional+kwargs signatures are
admitted; omitting the fallback fails at expansion with `kwargs-lower: missing argument :undefined`.
**Mandatory-ness needs no new machinery.**

## The wall is ONE-DIRECTIONAL

Rete ops are **ordinary wat functions**. Calling one outside a rule is odd, not illegal, and it keeps
working. What is illegal is the reverse: a non-conforming op inside a `where`.

Deliberate and load-bearing — it keeps the ops testable and composable as plain functions rather than a
walled garden reachable only through the engine, and "grow the expressivity" means adding ordinary
functions, not extending a special form.

## Strike order

| | |
|---|---|
| **S0** | **Rule the fork** (A vs B). Everything downstream depends on it. |
| **S1** | Re-audit T1's `total: true` entries against the ruled definition. Bounded, mechanical. |
| **S2** | Classify the 4 holon verbs (grounded per-verb; `dot`'s ±Inf reachability checked). |
| **S3** | Mint the fallback-carrying ops — `i64`, `f64`, `first`/`nth`, `string::subs`. **Native.** |
| **S4** | Give the already-total families their rete presence (law A only). |
| **S5** | Migrate the 39 refused corpus rows. |
| **S6** | **ARM** — the third conjunct at the fence. Never before S5. |

**S3 must be NATIVE, not wat wrappers.** A wat `defn` wrapping `i64::/` gets walked by `classify_fn`,
which sees the partial core op in its body and classifies the wrapper non-total — it would fail the
very fence it exists to satisfy. Implement as Rust intrinsics returning the fallback where core raises,
classified `total: true` directly.

## STOPs

- **⛔ The `_` wildcard.** `check.rs:5700`'s exhaustiveness error offers `"(or include `_` wildcard)"`.
  The `_`-arm-on-an-enum ban is doctrine whose checker rule is **unbuilt**, so nothing will stop a
  rider taking it. Taking it is a rejected strike.
- **Do not arm before S5.** A refused `first` with nowhere to go locks a user out of arithmetic.
- **Do not claim the speedup.** Step 0 of the compiled-where stone owns that number.
- **Do not design a fallback for an unreachable arm.** Cosine's NaN is already guarded; prove
  reachability before adding `:undefined` to anything.
- **No IO, spawn, service or mutation ops enter this vocabulary**, ever. The boundary is a rule
  *condition*, not general computation.

## Open — the builder's

1. **The fork (A vs B)** — the S0 blocker.
2. **`rete::holon::cosine` and the guarded `0.0`** — sentinel or acceptable answer?
3. **Does `=` on f64 belong at all?** Float equality is a hazard independent of NaN.
4. **The ingested-NaN limit** — accept as stated, or is there a wall for it?
5. **Naming under law A** — `:wat::rete::i64::+` is faithful to FQDN-always but long for a form written
   constantly. A `where` is a DSL binder context, and today's ruling was *bare means scoped* — so
   short names inside a `where` may be legitimate rather than a violation. Worth its own thought.
