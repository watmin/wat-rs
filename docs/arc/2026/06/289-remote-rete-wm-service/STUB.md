# Arc 289 — persistent remote rete working-memory service (STUB, banked)

**Status:** STUB / banked. Future arc. Captured 2026-06-19. Builder: *"a thing that's haunted me for years —
a remote HTTPS service that hosts a persistent working memory; you call fire-rules on it as a global blocking
thing, shoving facts in and selectively firing; I feel we have the foundation."*

## The vision
A long-lived **stateful rete server**: a working memory that persists across requests. Remote callers
`insert` facts, selectively `fire-rules` (a global blocking op), and `query` what derived — over the wire. A
managed, live inference endpoint other services talk to.

## The foundation that already exists (the haunting part — it's mostly built)
- **Persistent WM = a `defservice` holding a `Session` as state.** The engine stays PURE (`fire-rules` =
  `fn(facts × rules)` → frozen Session, arc 278); the *service* owns `{facts, rules}` across requests and
  threads the new Session back into itself. Statefulness lives in the actor shell, not the engine.
- **"Global blocking fire-rules" = the lockstep actor** ([[project_lockstep_blocking_channels_fpga]]). A
  `defservice` handles one request at a time, request→reply, blocking — the systolic model. Concurrent callers
  serialize; single-writer consistency by construction. `fire-rules` returns when the closure is computed.
- **Ops** = `insert` / `fire-rules` / `query` as service messages (lockstep calls).
- **Wire** = EdnRepresentable (arc 280) — facts/rules/queries as EDN.
- **EXPLAIN over the wire** = arc 278 P12: a caller can ask "why did this derive" and get the `DerivationNode`
  tree back (opt-in, re-derived from the stored `{facts, rules}`).

## Continuity + retraction (the two parts that feel hard — both already solved/built)
- **Continuity = the actor holding the `Session` as state.** The gen_server loop `handle(msg, state) →
  (reply, state')`; the `Session` IS the state, threaded forward each request (`insert`→Session', `fire`→
  Session'', `query`→read). The engine is PURE within a fire; the service is STATEFUL across fires — they
  coexist because statefulness lives in the actor's `state`, not the engine. Nothing new to invent: it's
  `defservice` threading `Session → Session'`.
- **Retraction = ALREADY BUILT (arc 278, oracle 4c + native P4c).** `retract` = remove-the-fact + re-fire;
  because `fire-rules` is **pure replay**, consequences vanish transitively + precisely with NO support-graph
  bookkeeping. The hardest thing in textbook RETE (TM / justification graph / Clara's hazard #1) **falls out of
  purity**. In the service it's just a `retract` message → new facts → re-fire.
- **It's a DEDUCTIVE db**: INSERT=`insert`, DELETE=`retract`, SELECT=`query`, materialize/trigger-cascade=
  `fire-rules`. The differentiator from a passive store: it INFERS (rules derive new facts on fire).
- **The one real perf choice**: v1 = **pure replay** (re-fire from facts each time — correct, simple,
  retraction free). A HOT persistent WM eventually wants **incremental** insert (P4b delta) + **incremental TM**
  (the support-store cascade). The breadcrumb named exactly this: the support-store cascade *"earns its place
  ONLY in a future persistent/streaming engine where memories live across fires."* **This service IS that
  engine** — it's where the incremental TM cut from the pure oracle returns as the perf path.

## The one genuine gap: the HTTPS / TCP+TLS transport
Today comms is **UDS** (+ the arc-272 rendezvous capability, SO_PEERCRED gating). HTTPS = TCP + TLS — a NEW
transport leg. Work: a TCP listener + TLS + HTTP framing on the comms trait (the `EdnRepresentable` Peer
abstraction should layer over it), then bind the rete `defservice` to it. Everything above the socket exists.

## Distinct from arc 287 (don't merge)
- **287 (WorkQuery v2)**: *ephemeral* pull-query — fire a transient rule over a frozen `{facts, rules}`, get an
  answer set, done. Stateless per query.
- **289 (this)**: *live stateful* WM — the working memory persists and accumulates across requests; you mutate
  it over time. Same engine + service substrate; different lifecycle (long-lived vs per-query).
- They compose: a 289 server could expose 287-style queries against its live WM.

## Open questions (decide at arc-open)
- HTTP semantics: REST-ish (POST /facts, POST /fire, POST /query) or a single EDN-RPC endpoint? (The builder
  dislikes SQL/ceremony — likely an EDN-RPC over POST.)
- Auth/capability over TCP (UDS had SO_PEERCRED; TCP needs a token/mTLS capability model — ties to the ocap
  doctrine).
- WM durability: is the persistent WM also snapshotted to disk ({facts,rules} blob, R5) for restart/triage?
- Backpressure / fire-cost: a `fire-rules` on a large WM blocks all callers — bounded by the actor; consider a
  per-fire budget or an async "fire-and-notify" (⚠ but NEVER fire-and-forget the channel — lockstep doctrine).
- The DDoS/anomaly face: the same server ingesting a packet/request stream + rete-rules ∪ VSA-similarity =
  detection-as-a-service. The career-long anti-botnet endpoint.

## Relations
- Built ON: arc 278 (rete engine), `defservice`/peers (214), arc-272 rendezvous, arc 280 (EdnRepresentable wire).
- Needs: a TCP+TLS/HTTP transport leg (the only new substrate).
- Composes with: arc 287 (query surface over the live WM), arc 288 (pretty-printed EXPLAIN responses).
- NOT a now-thing. Open after 278 closes + 280 ships the wire.
