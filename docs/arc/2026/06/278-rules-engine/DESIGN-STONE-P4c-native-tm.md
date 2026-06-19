# DESIGN — Stone P4c: native truth maintenance (retract = replay; the support-store cascade is CUT)

P4c was planned (PERF-ARC P4) as *"delta retract + TM cascade via the support store + the token `matches`
chain"* — the incremental analog of P4b for retraction. **Grounding the premise dissolved the build:** in the
engine's actual surface, native truth maintenance is already correct AND fast, with no new code. P4c ships as a
**differential gate + an affirmative cut**, not a feature.

## The grounding (probe-proven, not asserted)
- `:wat::rete::retract` is **engine-agnostic**: it removes a fact from `Session.facts` by value (stage-only),
  returning a new `Session`. It touches no memory and no engine path (`wat/rete.wat:1026-1047`).
- The native engine is **value-semantics / transient-within-fire** (the arc decision: `Session` in, `Session`
  out; memories are transient inside one `fire` and do not persist across calls). So `fire-rules'` (P4b)
  rebuilds the closure from `Session.facts` on every call.
- Therefore **TM falls out of replay** ([[project_rete_inserts_only_replay]]): `retract` (drop from facts) +
  `fire-rules'` (re-derive from the reduced facts) → the retracted fact's consequences are simply not
  re-derived — transitively and precisely. This is the *same* mechanism the wat oracle uses (stone 4c), now on
  the native delta engine.
- **P4b already made the replay linear.** The thing the support store was meant to optimize — avoiding a full
  re-fire on retract — is moot when a full re-fire is itself O(closure), linear, and is what every `fire-rules'`
  call already is.

Proof: `tests/probe_arc278_P4c_native_retraction.rs` runs the 4c scenarios on BOTH `fire-rules'` and the oracle
`fire-rules` and asserts native == wat — drop, transitive cascade, and precise (independent derivations
survive). 3/3 green. Native TM is correct, scenario for scenario.

## The cut (affirmative, four-questions)
**The support-store + `matches`-chain incremental retract cascade is CUT from the value-semantics surface.**
- **Obvious?** YES — a support store that lets retract remove only the affected tokens is *only* observable if
  memories PERSIST across `fire` calls. They don't (transient-within-fire). With a rebuild-each-fire engine, a
  support store is dead weight: it would be built during a fire and thrown away at the freeze boundary.
- **Simple?** YES, by removal — no provenance graph, no `matches`-chain bookkeeping, no retract-cascade code.
  Retraction is `retract` (exists) + `fire-rules'` (exists). Zero new engine surface.
- **Honest?** This is the point — the inserts-only/value-semantics model makes TM a property of replay, not a
  subsystem. Claiming we "built incremental TM" when replay already gives it would be the dishonest move.
- **Good UX?** YES — `retract` then `fire` is the same two-verb shape as the oracle; the user surface is
  identical for both engines; no new concept.

## Where the future-work tracks (exigere — named, not silently dropped)
The support-store incremental cascade earns its place in exactly ONE future surface: a **persistent / streaming
engine** where a long-lived session keeps its memories live across many `insert`/`retract`/`fire` cycles and
each operation must be O(delta), not O(closure) — the line-rate path for HTTPS/sampled-packet streams where you
do not want to re-fire the whole closure per packet. That engine is a **separate arc** (cross-fire persistent
memories — explicitly deferred in DESIGN-STONE-P4b §out-of-scope), not part of 278's value-semantics close. If
and when it is built, the support store + `matches` chain (CLARA-REF §3) is its retract path. Until then there
is nothing to build; this DESIGN is the record of why.

## Files
- `tests/probe_arc278_P4c_native_retraction.rs` — the native-TM differential gate (3/3, native == wat).
- No `src/` change. No oracle change. No `WorkingMemory` change.

## Out of scope = REJECTED / DEFERRED
- The support-store / `matches`-chain incremental cascade — **CUT** here (see above); tracks to the future
  persistent-streaming-engine arc.
- Public `(:wat::rete::fire)` wiring + bench vs Clara — **P5** (the last stone; 278 closes there).
