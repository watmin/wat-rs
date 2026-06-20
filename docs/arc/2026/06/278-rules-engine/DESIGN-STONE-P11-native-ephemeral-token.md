# DESIGN — Stone P11: the support chain as a cheap property graph (rete = net, twice over)

Dominate the last Clara cell (fan-out 40k: 134ms vs 96 = 1.4×) by making the support chain a **cheap property
graph** instead of per-token `Value` machinery — the SAME structure that is the operator diagnostic "how did
this fact get derived." Not the dead P10 (drop `matches`); the opposite: keep the chain, make it the graph.

## The shape (the framing that decides the repr)
Forgy's *rete* is Latin for **net**: the matching network. The provenance is a net too — so making it a graph is
the engine being honest to its name *twice over* (a net that builds a net). The natural form is a **property
graph**: **nodes = facts (data); edges = conditions** (the `alpha_id`/gate a supporting fact satisfied). Our
support entry already IS that edge:
```
token.matches = [(f1, alpha1), (f2, alpha2)]   for the token deriving F
                  └─node──┘ └edge┘             ← (supporting fact, condition/gate)  →  F
```
F's incoming condition-labeled edges = the firing. No explicit activation node (Clara inserts one; we put the
condition on the edge — cleaner, reference-for-functionality-not-parity).

**Pure, in-memory, no transaction tax.** This is the Cypher/graph model the builder reached for at AWS, freed of
the machinery that made him abandon Neptune (forced ACID — every insert a transaction — thrashing a
transaction-free app). Our graph is **deferred computation** (R5): built by re-firing a pure function, walked,
dropped — never a transactional write, never persisted. Neptune's data model, not Neptune; a net, not a database.

## The guiding light (the invariant this stone is gated by)
The support chain — `matches`, the `(fact, alpha-id)` edges — is the substrate of "how did this get derived" (the
P10 reversal made it non-negotiable). **This stone keeps the chain BUILT every fire and WALKABLE.** The perf win
is from a cheap *container* (no `Value` wrappers) and from *not persisting* it (regenerates on re-fire), NEVER
from dropping it.

## The residual (grounded)
Per token today: `support = Value::Tuple(Arc::new(vec![fact, i64]))` (Vec+Arc) + `matches.push_back` (VectorSync
node) + `make_token` boxes `struct_form: Arc::new(vec![PV, PM])` (Vec+Arc) → ~6 allocs × 40k; the bench
(`fanout-join.wat`, times `fire-rules'` incl. final freeze) pays all of it. The fact itself is *already* an `Arc`
pointer (`Value::Record` = `Arc<String>`+`Arc<Vec>`), so `fact.clone()` is a refcount bump — the waste is the
**wrappers** we rebuild around it per token, not the fact. Drop the wrappers; keep the pointer.

## Why beta can be ephemeral (grounded)
- Differential = **observable equivalence**: `query(fired, T)` derived-fact counts, NOT raw Session equality
  (`probe_arc278_P4a:5-6`; deep-cascade asserts the deepest count). Beta tokens are never compared.
- `query` reads production-memory; `retract` re-fires; `fire_once` clears memories at start (`kernel.rs:798-803`)
  → a stale frozen beta is never consumed. The `beta-memory` reads in `rete.wat` (`:995,:1018,:1057`) are
  oracle-internal (`fire-fixpoint`/`fire-rules-spec`) or `retract` passthrough — none read a native-fired beta.
- Round-trip test asserts `to_persistent(to_transient(fired)) == fired` only — empty round-trips to empty; never
  asserts fired-beta non-empty (`kernel.rs round_trip_fired_session`).

## The contract (LOCKED) — the build scope of THIS stone

### Native token = a cheap graph node + its condition edges
```rust
struct Token {
    matches:  Vec<(Value, i64)>,                    // the condition-labeled edges — KEPT, cheap owned Vec push
    bindings: rpds::HashTrieMapSync<Value, Value>,  // UNCHANGED rpds — matcher.rs resolve_operand untouched
}
```
- `matches` = the edges (cheap + walkable, serves the guiding light). `bindings` stays rpds so `production_pass`
  → `build_insert_fact` reads it directly (no matcher.rs change, no per-firing conversion).
- `WorkingMemory.beta`: `HashMap<i64, Vec<Value>>` → `HashMap<i64, Vec<Token>>`.
- **Element stays `Value::Record`** (alpha memory, low volume — not the residual). `root_join_pass` seeds a
  native `Token` from each Element.
- The passes (`root_join_pass`, `hash_join_pass`+`keyed_join`, `extend_token`, `production_pass`) operate on
  native `Token`; `extend_token` pushes `(fact.clone(), alpha_id)` onto `matches` (Vec push) + folds bindings.
- Freeze: `to_transient`/`to_persistent` stay **lossless** (Value↔native Token, for the round-trip contract).
  `fire_once_session` / `fire_fixpoint_delta` **`wm.beta.clear()` before `to_persistent`** — the fast path drops
  the 40k tokens uncconverted (the result is in production-memory). THIS is the win.

### Out of scope = rejected
Oracle (`fire-rules-spec`) UNCHANGED. No `Value` variant changes (no megafile ripple). `matcher.rs`
`resolve_operand`/`build_insert_fact` UNCHANGED (bindings stay rpds). Element stays `Value`. Bindings→SmallMap
(possible Stage B only if 40k doesn't close). **The walk renderer + the fact→token entry index — the NEXT stone
(P12 / DESIGN-STONE-S), not this one.**

## The named follow-on (the walk — P12 / EXPLAIN, not this stone)
This stone makes the graph cheap and guarantees the edges survive. The *walk* — "how did this get derived" —
needs one more thing: the **fact→producing-token entry index** (the 4c cut, re-introduced) so you can enter the
DAG from a derived fact, follow its incoming condition-edges to supporting facts, and recurse to inputs. Built in
an **explain-retaining fire** (does not clear beta + builds the index); the fast fire stays lean. The
"which-gate-misfired" overlay = the static rule-structure (structured `Condition`, DESIGN-STONE-S) ∧ the runtime
edges. Named here; built next.

## Acceptance (three gates — all must hold)
1. **Differential GREEN**: `deep_cascade` (10/20), `P4a`, `P2`, `P4c`, `northstar`, the fan-out differential, lib
   `rete` (incl. round-trip), full lib + `test`. Any RED → STOP.
2. **Guiding light GREEN**: a probe (Rust unit test in `rete::kernel`) that runs the passes on a small cascade
   and asserts a production-reaching token's `matches` carries the expected `(fact, alpha-id)` edges (N for an
   N-condition rule). Empty/lossy chain → STOP (severity = a differential RED).
3. **The win** (orchestrator runs): `fanout-join.wat` at 40k drops below Clara's ~96ms (from 134); deep-cascade +
   16k/20k do not regress.

## STOP triggers
Any differential RED · the support chain goes empty/lossy (guiding-light breach) · reaching to touch the oracle /
`Value` / `matcher.rs resolve_operand` / Element / the walk-index (next stone). Halt and surface.
