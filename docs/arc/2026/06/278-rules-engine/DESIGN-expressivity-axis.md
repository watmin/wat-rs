# DESIGN — the grid's third column: EXPRESSIVE, beside CORRECT and FAST

> **Origin (builder, 2026-08-01):** *"i wish to ensure that our rete is expressive, correct and fast
> — where is our rete now?… i believe we have won in correct (parity check) and fast (perf check)
> but not expressivity."* And the bar: *"my objective was clara superiority in a pure deployment.
> clara has impure capabilities we will not build — but for where purity matters, i want to be
> superior."*

## The reframe this rests on — expressivity BOUNDS the other two

We had 27/27 `:accuracy :match` and 27/27 `:us`. Both were true. Both were measured **inside a
fence nobody knew was there**: the nine axes use generic `>` everywhere and have never once used a
String verb in a `where`, because anything else panicked at the purity fence (`0d439a55`).

A correctness result and a perf result are claims about **the set of programs you can write**. That
set just grew by 35 verbs plus the VSA seam. So this is not a third axis standing beside the other
two — it is the axis that **says how much the other two cover**. It goes first for that reason.

## The three rulings (builder, 2026-08-01) — the whole shape of the instrument

**① `:parity` is binary, and it IS accuracy.** *"we either have it or we don't - this is the same as
accuracy."* No new verdict vocabulary, no tri-state rubric. A shape is expressible-and-correct, or
it is not. The grid already emits `:accuracy :match`; expressivity rows emit the same thing.

The surplus/cut distinction does NOT become a verdict column — it becomes **which rows exist**:

| situation | how it appears | why |
|---|---|---|
| both engines express it | a row with a Clara counterpart; `:match` required | parity |
| only we express it (holon) | a row with **no Clara side**; oracle differential only | *"clara doesn't have holon so it cannot express that… we should prove holonic ops do work"* |
| only Clara expresses it, we CUT it | **not a row.** A scope statement in this doc | salience, `insert!`/`retract!`/`insert-unconditional!` — refused because impure. A test would score a decision as a failure |

**② The oracle is the anchor.** *"we must answer the same as our oracle."* Every row runs the
standing differential — **native == the wat oracle**, bit-for-bit — and *additionally* compares to
Clara where Clara can express the shape. This is what makes the holon rows meaningful with no Clara
side at all: `ANCORAM NON AMITTIMVS`, the oracle never leaves.

It also settles a weaker design I floated and should not have: a row asserting only "this compiles"
is the vacuous-gate class (R59 — a green that passes whether or not the mechanism works). A shape
that compiles and derives the wrong facts must go red. **Every row fires and compares derived sets.**

**③ The row list comes from the verbs we HAVE, plus a named gap analysis.** *"the funcs we have now
and we need to identify what we don't have."* Not a hand-written list of shapes — that is the corpus
census a fourth time. The source is `dispatch_keyword_head_value`'s arms (221 today), joined against
the purity classification, so a verb minted tomorrow appears as an **unproven row automatically**.

## What it emits

The grid's existing `#grid/Result` contract, one row per shape, reusing `:accuracy` verbatim:

```clojure
#grid/Result {:axis "expressivity" :shape "string/starts-with?"
              :expressible true :accuracy :match :clara :match  :derived [...]}
#grid/Result {:axis "expressivity" :shape "holon/coincident?"
              :expressible true :accuracy :match :clara :n-a    :derived [...]}
#grid/Result {:axis "expressivity" :shape "apply"
              :expressible false :accuracy :n-a  :clara :match  :derived []}
```

- `:expressible false` with `:clara :match` is the **red row** — Clara can say it, we cannot. That
  is the only shape of failure this axis reports, and it is exactly the bar the builder set.
- `:clara :n-a` is our surplus, and it must carry a real `:accuracy :match` against the oracle or it
  is a claim with nothing behind it.

## Scope — the affirmative cuts, recorded so a test never scores them as gaps

Deliberately NOT expressible, by design, because they are impure
(`[[project_rete_inserts_only_replay]]`): **salience**; **`insert!` / `retract!` /
`insert-unconditional!`** (the mutating bangs); **arbitrary fact types**. Clara has them. We refused
them, and the refusal is what buys `RENASCOR NON RETRACTO` — the pure oracle Clara structurally
cannot have. These are a scope statement here, never a row.

## The sibling stone this depends on, and it is the extirpare rung

**A purity-completeness gate.** Every verb in the dispatch table is classified pure, impure, or
**deliberately-unclassified-with-a-reason**; a new verb with no classification fails the gate. It
reads the same list the matrix walks, so neither can go stale while the other holds. `0d439a55`
closed 35 verbs by hand; this is what stops the 36th recurring silently. Root remains arc 255's
registry (purity declared where the verb is *defined*); the gate is buildable now and would have
caught every one of the 35.

## Where the rows come from — the join, concretely

1. Enumerate `dispatch_keyword_head_value`'s `:wat::core::` / `:wat::holon::` arms.
2. Join against `intrinsic_meta`'s classification.
3. **pure ∧ classified** → a row is REQUIRED. Does a `where` using it compile, fire, and match the
   oracle?
4. **deliberately-unclassified** → a row asserting the *reason still holds*, not a capability.
5. **unclassified, no reason** → the gate fails. This is the state that must not exist.

`wat-scripts/scratch-pad/probe-where-shape-spread.wat` (`0d439a55`) is the seed: nine rules spanning
arithmetic, accessor, nested accessor, string, collection, map, user-fn, multivar+deep, bool. **Five
of the nine could not compile before the purity fix.** It grows into the axis; it is not a parallel
instrument.

## ⚠ This EXTENDS the grid; it does not become a second grid

The grid already has the harness — nine axes, a Clara-side generator (`gen-*.sh`), the
`#grid/Result` contract, `run-axis.sh` with both fire and wall clocks. Building expressivity
standalone would create precisely what `UNADOPTED.md` exists to catch: a capability with no consumer
but itself. **New axis in `wat-scripts/perf/grid/`, same contract, same runner.**

## What re-earning the other two columns requires

Honest consequence of the reframe, stated so it is not quietly skipped: the existing nine axes were
authored inside the fence. Once the newly-legal shapes are expressible, **the accuracy and perf
claims need re-establishing over the wider surface** — not because the old runs were wrong, but
because their boundary was the fence
(`[[feedback_a_measurements_boundary_is_its_claims_boundary]]`).

## Out of scope = REJECTED

- **The compiled-`where` stone (#49a) and the discrimination tree (#49b).** Parked. Both design
  against "which shapes are legal", and that set just changed; the per-shape decomposition runs on
  this axis's fixture once it exists.
- **Task #50** (the filter loop's token clone) — independent, and 11% of one phase.
- **Any new verdict vocabulary.** Ruling ① — `:accuracy` verbatim.
- **Classifying the remaining holon verbs.** The four are ruled; the threshold siblings and the
  learning ops are named in `purity.rs` and want a ruling, not an assumption.
