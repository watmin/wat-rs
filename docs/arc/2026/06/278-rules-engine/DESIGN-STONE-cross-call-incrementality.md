# DESIGN-STONE — cross-call incrementality: a re-fire must cost the DELTA, not the BASE

> **Status: RULED, NOT BUILT.** The builder, 2026-08-01, choosing between accepting the cost and
> fixing it: *"make it real — cross-call incrementality: the session keeps its memories warm and a
> re-fire processes only the delta… that's what would make `session.insert(facts)` persisting across
> blocks actually pay off rather than just work."*
>
> Nothing here is scheduled. Expressivity measurement continues; this is the tracked stone.

## What forced it — the measurement, not a reading

The builder's session model:

```ruby
session = make_session(rules)
session.insert(persistent_facts)      # a committed BASE

with_session(session) do |s|          # an OVERLAY
  s.insert(temp); s.fire_rules; s.query(...)
end                                   # temp facts gone, base untouched
```

`NOTE-overlay-read-path-and-distributed-horizon.md` (2026-06-20) already settled that the **isolation**
is free: value semantics over persistent collections means the base is protected by construction and
the discard costs nothing. What it did not measure is the **cost of the block's fire**.

Measured (`wat-scripts/scratch-pad/probe-overlay-refire-cost.wat`, two runs, native path —
`fire-rules` delegates to `fire-rules'`, the Rust delta kernel, `wat/rete.wat:1910`):

| base N | cold fire | **no-op re-fire** | overlay (+10 facts) | noop/cold |
|---|---|---|---|---|
| 1000 | 1.18 / 1.75 ms | 1.14 / 1.58 ms | 1.26 / 1.52 ms | 0.97 / 0.90 |
| 2000 | 2.55 / 2.82 | 2.23 / 2.40 | 2.48 / 3.40 | 0.87 / 0.85 |
| 4000 | 6.53 / 10.0 | 5.82 / 9.96 | 6.90 / 6.48 | 0.89 / 1.00 |
| 8000 | 11.2 / 11.3 | 9.44 / 9.73 | 10.1 / 9.26 | 0.84 / 0.86 |

The load-bearing column is the **no-op**: firing an already-fired session with **nothing added** costs
84–100% of a cold fire and scales with the base. If the engine tracked what it had already derived,
that would be flat and near-zero.

**`fire-rules` is idempotent in RESULT but not in COST.** The semi-naive delta operates *within* one
fire (across cascade rounds); it does not carry *across* fire calls.

Not a discovery — `PVRITAS VERVM, NON CELERITATEM` already owns the class (*purity bought correctness,
not performance*) and R22 `OCVLI NOVI, ORACVLVM IMMOTVM` lists incremental insert/TM as targets that
were never built. This is the first measurement of it in the shape the session model needs.

## ★ THE HARD PART, and it is not performance — the current cost is BUYING correctness

Read this before drawing anything, because the obvious implementation is wrong.

**Today's O(everything) re-fire is what makes non-monotone negation safe.** Every fire recomputes
from `{facts, rules}`, so stratification runs fresh over the *complete* fact set — a rule that asks
"is T absent?" always sees a finished T (`NEGATIO COMPLETVM POSCIT`).

Now make insert incremental and re-run R18's exact bug. R18 `RENASCOR NON RETRACTO`:

> `Ok2`, derived in round 1 when `Bad2` didn't yet exist, **persists in the facts and is never
> retracted** — non-monotonic negation over a monotonically-growing fact base.

Stratification fixed that *within* a fire. A cross-call incremental insert reintroduces it *at the
fire-call boundary*: the base fire derives `Ok(x)` because nothing made `x` bad; the block inserts a
fact that makes `x` bad; an incremental fire that only processes the delta never revisits `Ok(x)`,
and the answer is silently wrong.

So the stone is **not** "skip work." It is: **skip the work that provably cannot change, and
re-derive everything a negation or accumulator could invalidate.** That is why R22 listed incremental
TM as its own target rather than folding it into incremental insert — it is the harder half, and it
is a *correctness* change wearing a performance change's clothes (the same warning `#49a` carries
about its part (b)).

Monotone-only rules are the tractable subset. Negation and accumulators are where this gets decided.

## What the stone must deliver

1. **The delta cost is the delta's.** A no-op re-fire is ~free; an overlay fire scales with the facts
   added, not the base.
2. **The oracle does not move.** All of it lands in the Rust kernel; `fire-rules-spec` stays the naive
   pure-replay reference (R22 — *the kernel may diverge in SHAPE while converging in RESULT, because
   the oracle checks*). The differential is what makes an aggressive kernel change safe.
3. **Negation and accumulators stay correct across calls**, or are explicitly and loudly excluded from
   the fast path rather than silently wrong.
4. **Value semantics survive.** A `Session` is still a value; a warm memory must not become shared
   mutable state that makes two holders of "the same" session observe different things.

## The gate already exists, and it is RED

`wat-scripts/scratch-pad/probe-overlay-refire-cost.wat` is the acceptance test, written before the
stone. Today it reports `noop ≈ cold`. When the stone lands it must report `noop` flat and near-zero
while `cold` still scales — and its non-vacuity assertion (the overlay must derive exactly `+10`)
already fails loudly if the overlay stops deriving anything.

Plus, non-negotiably: the full differential (`native == oracle`) and the Clara grid, both unmoved.

## What is NOT grounded, and must be the first act

**The mechanism.** The measurement says *what the cost does*, not *why*. Whether the kernel rebuilds
memories from `facts` on entry, ignores the memories it was handed, or re-walks the alpha network
regardless — none of that is established. Ground it by reading `fire_rules_prime` / the kernel's fire
entry **before** designing, because the fix differs completely between "the memories are discarded"
and "the memories are kept but not consulted."

Do not skip this. The record's most repeated failure this arc is attributing a cost to a component by
reading it, and the second is designing against a mechanism nobody checked.

## Kin

- `NOTE-overlay-read-path-and-distributed-horizon.md` — the isolation half (free); its
  `:what-if`/`add-and-fire` design says *"zero new substrate, pure composition"*, which is **true for
  correctness and false for cost**. This stone is what makes that sentence true on both axes.
- `PVRITAS VERVM, NON CELERITATEM` — purity buys correctness, not incremental cost.
- R18 `RENASCOR NON RETRACTO` + `NEGATIO COMPLETVM POSCIT` — the leak this must not reintroduce.
- R22 `OCVLI NOVI, ORACVLVM IMMOTVM` — the oracle unmoved; T2/T3 named and unbuilt.
- `#49a` — the sibling shape: a semantically-inert stone first, the semantic one on a proven base.
