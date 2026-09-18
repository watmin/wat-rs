> # ⛔⛔ SUPERSEDED — DO NOT EXECUTE (2026-06-19). The builder made operator-facing EXPLANATION
> NON-NEGOTIABLE: *"being able to print and debug an activation chain is tremendous diagnostics — handing an
> operator a concrete explanation for a service decision is non-negotiable."* `matches` **IS the activation
> chain** — the `(fact, alpha-id)` support tuples are the provenance of *how* a token was derived (which facts,
> through which gates). It is NOT dead weight; it is the substrate for the why-derived / which-gate-misfired
> diagnostic. **KEEP `matches`.** This whole "drop it" design is wrong — the grep showed no *current* consumer
> only because the explanation RENDERER isn't built yet (a missing capability, not dead data). The corrected
> direction (see breadcrumb): (1) dominate 40k by a CHEAP support REPRESENTATION (compact, not
> `Arc<Vec>`/`PV<Tuple>` per support) that PRESERVES the chain; (2) BUILD the activation-chain explanation
> renderer that reads it. Kept on disk as the reasoning + its correction (we almost deleted a non-negotiable).

# DESIGN — Stone P10: annihilate the dead support-chain provenance (dominate fan-out-40k) [SUPERSEDED]

The last cell where Clara leads (fan-out 40k: 134ms vs 96ms) is **not** JVM-beats-Rust — it is us paying for a
data structure **nothing reads**. Studying the lair (slow is smooth) found the dragon's weak point: it is dead code.

## The weak point (grounded, not reasoned)
Every consumer of a token's `matches` (the `(fact, alpha-id)` support chain) feeds it straight back into
`extend_token` — `kernel.rs:618→629` (batch hash-join), `:1306`, `:1330` (delta hash-join). **Every other
reader ignores it** (`_`): production firing (`:777, :1254, :1346, :1389`), every join probe (it takes only
`bindings`). The oracle reads `Token/matches` in exactly one place (`rete.wat:689` — the same extend).

So `matches` is **pure provenance with no consumer**: built → carried forward by `extend_token` → frozen into
the Session → **never read** for `query`, for production firing, or for truth maintenance (TM is replay —
`retract` + re-fire; proven by `probe_arc278_4c_retraction`, which never touches `matches`). In the replay-TM
engine it is dead weight. The cost we pay for it, per token: `Value::Tuple(Arc::new(vec![fact, i64]))` (a heap
Vec + an Arc) **plus** the `VectorSync` new+`push_back` — ~3–4 allocations × the join cardinality (×40k at the
extreme). Clara carries provenance too, but cheaply; we carry it expensively and never look at it.

**The earlier rejection was under-grounded** (a documented correction). "Drop `matches`" was waved off as
"ungated + un-does the streaming engine." Grounding both: it IS gated (the `query` differential is the net; 4c
proves TM is replay-based, independent of `matches`), and it does NOT un-do the streaming engine — that future
cross-fire engine builds its OWN incremental support store when it exists (`exigere`: we do not carry dead
provenance speculatively for an unbuilt consumer; we name where it tracks). Dead-weight removal, gated.

## The kill (`src/rete/kernel/fire/` — the native fire passes only)
Stop POPULATING `matches` in the native engine. The `bindings` (the live data — drives firing) is untouched.
- `make_token` callers in the fire passes: build the token with an **empty** `matches` PV.
  - root-join seed (`:489-492` batch, `:1197-1200` delta): drop the `support` Tuple + the `matches_pv` new/push;
    seed `make_token(empty_pv, bindings)`.
  - `extend_token` (`:551-567`): drop the `support` Tuple build + `tok_matches.push_back`; the produced token
    carries the (empty) matches through unchanged. Its signature can drop the `tok_matches`/`el_fact`/`alpha_id`
    params it only used for the support, OR keep them and ignore — pick the clean cut.
- The `matches` field STAYS on the Token record (struct_form position 0) — it is simply always an empty PV in
  the native engine. No type change, no struct_form arity change.
- `to_transient` / `to_persistent` / `pm_to_hashmap` / `hashmap_to_pm`: **UNCHANGED** — they remain lossless;
  they preserve whatever `matches` a token carries (empty, for native). The round-trip test
  (`round_trip_fired_session`, which fires via the now-native `fire-rules`) round-trips empty consistently.
- The wat ORACLE (`fire-rules-spec`, `rete.wat`) is **UNCHANGED** — it keeps full `matches` (it is the spec).

## Why it's safe (the contract)
- **Observable behavior unchanged.** `query` reads production-memory (derived facts), never beta `matches`.
  Differential (`probe_arc278_deep_cascade` 10/20, `…P4a`, `…P2`, acceptance 2b–5a, north star) stays green.
- **TM unaffected.** 4c retraction is replay (`retract` removes a fact, re-fire recomputes) — it never reads
  `matches`. 4c staying 4/4 is the proof.
- **No consumer exists** (grounded above). If the shadowdancer finds ANY test asserting `matches` *content*
  (provenance), that test checks dead data — STOP and surface it (do not weaken it silently).

## Files / out of scope
- `src/rete/kernel/fire/` — the fire passes (make_token seeds + extend_token). NOTHING else.
  (`make_token` has not existed since `82b9b5518`; the passes build `Token { … }` literals — see `seed_token_binds` in `fire/mod.rs`.)
- NOT the oracle, NOT `to_transient`/`to_persistent` (stay lossless), NOT `bindings`, NOT the Token type/arity,
  NOT `Value`. No new probe (behavior-preserving — the differential is the net; the bench is the win).

## Verify
- Differential family + 4c + round-trip all green (behavior + TM unchanged).
- The win (orchestrator runs): `wat-scripts/perf/matrix/fanout-join.wat` at 40k drops **below** Clara's ~96ms;
  deep-cascade + 16k/20k do not regress (they get cheaper too — every token in every fire stops paying the
  dead support cost). If 40k now leads Clara across the board → **the grid is ours; the realization stands
  unqualified.**

## exigere — the named future
When the persistent/streaming engine (cross-fire memories, O(delta) retract) is built, it re-introduces an
incremental support store (the `matches`/justification chain it actually consumes). Tracked there, not carried
dead here.
