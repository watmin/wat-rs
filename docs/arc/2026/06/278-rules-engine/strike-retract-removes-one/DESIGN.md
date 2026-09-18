# DESIGN — `retract` removes ONE occurrence

**Strike 2 of 2.** Rides on `strike-factbag-one-owner` (`09e3d912c`), which made `FactBag` the one
owner and named today's behaviour `remove-every-equal`. This strike replaces that door with
`remove-one`, lands the redrawn proof axis in the same commit, and deletes the old door.

## Why — and the argument does not need Clara

Driven at HEAD: `acc::count` over `F(0)` inserted **twice** returns **2**; inserted once, **1**.
Input multiplicity is **wat's own observable semantics**. So:

> **The engine can COUNT the duplicate but cannot REMOVE it.** `insert` takes the count 1→2;
> `retract` takes it 2→0.

The tree also has a written, reasoned position that input multiplicity is real — `fire.wat:234`,
on `retain-supported`: *"⛔ IT MUST NOT DEDUP. `insert$oracle` never dedups, so a caller that
stages the same fact twice **genuinely holds it twice**… a retraction with no cause."* And
`retract`'s own docstring claims *"Symmetric with insert"* and *"value-precise"* while removing N.

**It was never a decision.** `BRIEF-STONE-4c-truth-maintenance.md:46-50` pinned the implementation
as *"mirror `merge-facts`"* — and `merge-facts` is set-semantic for the **termination invariant**
(`fire.wat:214`), a reason that does not reach retraction. The set-ness came along with the fold.

Clara agrees (drops one), which the builder's hierarchy makes decisive — but the internal argument
stands alone.

## NOT in scope — do not "fix" these

- **Derived-fact multiplicity** (Clara 2, wat 1). CLOSED AS JUSTIFIED: set semantics on derived
  facts IS the termination proof (`fire.wat:214`, `:255`). Making derived facts a multiset removes
  the termination argument.
- **The TMS fuzzer.** ⛔ It has **no model** — `final-facts` exists only inside its header comment.
  `tms::step` calls the real `retract`, and `run-prog` folds it for all four arms, so the cure
  moves them together and the file stays green. Its `:26-30` comment asserting a replay the file
  does not contain gets **struck** — that is the only edit it needs.
- Insert-side dedup (the set-semantic alternative). Rejected: it would change `acc::count` from 2
  to 1 across every accumulate over input, and contradicts `fire.wat:234`.

## THE ONE CONTRACT DECISION

**`remove-one` drops the FIRST value-equal occurrence and preserves the relative order of every
other fact.** Absent ⇒ the bag is returned unchanged (no error, no signal). First-not-last so the
result is deterministic; order-preserving so `retain`'s sub-multiset property and the convergence
argument at `fire.wat:316-322` are untouched.

No `remove-first` primitive exists, so this is a fold carrying a "already dropped" flag. The house
idiom for fold state is a small record — `:wat::rete::StratifyAcc` (`stratify.wat:43`) and
`FireStratAcc` (`fire.wat:389`) are the precedents.

## Blast radius — measured, and it is one axis

Every `retract` call site was checked for duplicates in flight. **None has any:**

- `wat-scripts/perf/grid/{where-exists,where-not-fact,where-not-or}.wat` — `where-exists:91-92`
  nests two retracts over **distinct** `Wind` facts; the others retract singly-inserted facts.
- `tests/rete/probe_arc278_4c_retraction.wat` — parts B/C/D each seed Oslo ONCE and retract once.
- `tests/rete/probe_arc278_P4c_native_retraction.wat` — distinct facts, asserts precise TM.
- the three `differential-fuzz*` files — all arms share the verb.

So the cure is **behaviourally invisible everywhere except the redrawn axis.** That is a checkable
prediction, and it is the strike's main safety property.

## The redrawn axis — driven both engines before drawing

The shipped axis cannot see the defect: it reports `MISMATCH` at **zero retracts** (wat `[0 1 2]`
vs Clara `[0 0 1 1 2 2]`), because its seed doubles **every** key and so trips the justified
derived-multiplicity divergence. Its verdict has one possible outcome. See `../strike-retract-multiplicity-proof/REVIEW.md`.

**Redraw: duplicate ONLY the retracted key.** `F(0)×2`, `F(k)×1` for k>0, `G(k)×1`; retract `F(0)` once.

| | wat / oracle | Clara |
|---|---|---|
| before the cure (driven) | `[1 2]` | `[0 1 2]` |
| after the cure (expected) | `[0 1 2]` | `[0 1 2]` — **match** |

Sole witness is key `0`. The axis goes green with the cure and becomes the permanent regression gate.

## Landing shape

**Axis and cure in ONE commit.** The axis alone reds `ci.yml:262` (three-way, no args); the cure
alone has no acceptance test. The pre-cure RED is already captured verbatim in
`../strike-retract-multiplicity-proof/SCORE.md`, so the mutation proof exists on the record.

`CORRECTNESS_SIZES`' `want_n` moves **2 → 3** — that bump is itself a witness of the behaviour change.

## STOP triggers

1. Any test outside the redrawn axis changes value → STOP. The blast-radius measurement above says
   nothing else can move; if something does, the measurement is wrong and the strike must re-plan.
2. `remove-one` cannot be written order-preserving → STOP. `fire.wat:316-322`'s convergence proof
   depends on it; do not trade it for a simpler fold.
3. The three-way verdict after the cure is not `:accuracy :match :oracle-accuracy :match
   :port-accuracy :match` → STOP and report the raw `#grid/Verdict` line.
