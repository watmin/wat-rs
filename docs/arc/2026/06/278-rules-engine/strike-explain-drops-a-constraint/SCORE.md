# SCORE — D6, weighed against the orchestrator's own re-run

> **The cure lands and both gates were crossed. The sharpest finding is mine: I turned the floor RED
> and pushed it, by committing a `.wat` into a gated tree without running the gate.**

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ the enum constraint is IN the payload | ✅ `(:wat.rete.core.enum/= :d6.Grade/Hi :d6.Grade/Hi)` — **both** operands substituted |
| 2 | ★ i64 control unchanged | ✅ `(:wat.rete.core.i64/> 9 5)`, byte-identical in every payload |
| 3 | both gates crossed | ✅ mutation 2 REDs — see A |
| 4 | an unrenderable constraint is observable | ✅ a positional marker keeps `constraints.length` == the condition's constraint count |
| 5 | doc matches code | ✅ narrowed, and a **second** over-claim fixed — see D |
| 6 | `classify_constraint_head` untouched | ⚠ its *re-check* was **deleted as dead** — see C |
| 7 | engine untouched | ✅ zero diff under `src/rete/kernel/fire/` |
| 8 | lints | ✅ 210/210 |
| 9 | clippy | ✅ rc=0 |
| — | floor | ✅ **`5336 tests run: 5336 passed, 21 skipped`** (5332 + 4) — *after* I fixed my own red |

## ⭐ A — MUTATION 2 REDS, SO BOTH GATES WERE REALLY CROSSED

The named failure was *thread `sym` and stop*. Reverting **only** the `Value::Enum` arm, keeping the
`sym` thread:

```
FAIL a_unit_enum_constraint_reaches_the_explain_payload
[(:wat.rete.core.i64/> 9 5) (:wat.rete.explain/constraint-not-rendered :wat.rete.core.enum/= 1 "…has no literal spelling…")]
```

And it reports operand **1** — the *left*, bound operand. **My DESIGN's driven line said `b=false`,
implying the cure was about the right-hand literal.** Both operands resolve to `Value::Enum` and both
needed the arm; only one needed `sym`. A rider working from my line alone could have shipped an arm
guarded for the right-hand side only.

## ⭐ B — THE MARKER IS THE RIGHT SHAPE, AND ITS HEAD IS DELIBERATELY NOT CALLABLE

Part 2 landed as a positional marker rather than a refusal — correct for a debugging surface, where
failing the whole call to report one unrenderable operand destroys the nine things that worked. The
detail that earns it: **the marker's head is not a `RETE_OPS` row**, so a consumer that tries to
*evaluate* it fails by name instead of receiving a verdict on a comparison nobody performed. That is
the omission made un-mistakable rather than merely visible.

`constraints.length` now equals the condition's inline-constraint count **always** — that is the
property; the enum arm is the instance.

## ⛔ C — MY BRIEF SAID "THREE `continue`s"; ONE IS STRUCTURALLY DEAD

Verified by me: `ReteClauseShape::Constraint` has **exactly one producer** (`clause.rs:366-368`),
guarded `k if classify_constraint_head(k).is_some()`. So the payload builder's re-check
`if classify_constraint_head(op).is_none() { continue; }` **could never fire**. Two `continue`s can
drop a constraint, not three.

The rider deleted it *and* added `a_constraint_shape_implies_a_classifying_head`, asserting the two
classifiers agree **in both directions** — so if a second route to `Constraint` ever appears it reds
there, rather than the payload silently going short. Removing a dead guard while installing the
invariant that made it dead is the correct form of that move, and it mutation-proved the new gate on
**both** arms.

## ⛔⛔ D — I TURNED THE FLOOR RED AND PUSHED IT

`c9bb8044b` — my own strike-draw — committed the reconnaissance probe into
`wat-scripts/scratch-pad/`. **Two gates read every `.wat` under that tree, and I ran neither before
pushing.** The rider found it:

```
🔥 2 `:wat::rete::` name(s) are written in CODE under wat-scripts/ and resolve to NOTHING.
  wat-scripts/scratch-pad/d6-…​.wat:32  :wat::rete::DerivationNode/via
  wat-scripts/scratch-pad/d6-…​.wat:34  :wat::rete::DerivationStep/constraints
```

Reproduced by me. **My floor discipline is "run the gates before pushing", and I had run the floor at
`ab606b671` — before the commit that added the file.** A `.wat` landing in a gated tree is exactly
the case where "the floor was green earlier" is worthless.

**It is also a real lint gap**, and the rider was right not to paper it: those names are **live** —
the program runs — but record accessors are *synthesized at freeze* from the `defrecord` at
`wat/rete.wat:374` and never appear textually under `src/` or `wat/`. So the attestation half can
never see them, and **any `wat-scripts/` file touching a rete record accessor is unavoidably RED**.
None of the gate's three offered fixes applies: not a typo, not retired, and a
`rune:lint(rete-name-unminted)` would be a lie about a minted name. Tracked as **C15**.

The probe now lives with the strike that cites it, as arc record — which is what it always was.

## ⭐ E — STOP-4 WAS WIDER THAN THE TRUTH, AND ONLY DRIVING CLOSED IT

My STOP-4 asked for "any other `Value` variant that drops the same way", and `validate/typing.rs`'s
header reads like an open door (*"a `?var` and a literal are left alone"*). The rider expected
`PersistentVector` to reach the payload; it cannot — `ConstraintTypeNotComparable` walls it at
freeze. **Spec and code agreed and the inference from them was wrong; only driving it found that.**
The residue is exactly one case (tagged enum), and it is gated by a `.wat.bad` + golden rather than
asserted in prose — so if that wall is ever relaxed, the arm goes RED.

## ⚠ F — STOP-2 FIRED HONESTLY

The tagged-variant spelling **is** ambiguous — `(:d6t::Grade::Scored 7)` (a diagnostic renderer's
form, lossy by design) versus `#d6t.Grade/Scored [7]` (the reader form) — and neither is "the literal
the author wrote", because a tagged operand can only arrive bound from a fact field. The rider
refused to pick and routed it to the marker. **That is the STOP working as intended on a
user-visible decision.**

## Per-arm status

| arm | status |
|---|---|
| `sym` threaded; unit enum renders | **proven** (mutation 1) |
| `Value::Enum` unit arm | **proven** (mutation 2) |
| i64 control | **proven** — identical across all four payloads |
| tagged enum → marker | **proven**, driven |
| operand-did-not-resolve → marker | **reachable but NOT driven** at HEAD-with-fix — only under mutation 1. Disclosed by the rider rather than claimed |
| non-comparable operand → freeze wall | **not reachable** in the payload; driven as a `.wat.bad` |
| the dead `classify_constraint_head` re-check | **not reachable by construction** — deleted, invariant gated |
