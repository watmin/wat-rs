# DESIGN — Stone 4b: cascade-to-fixpoint (derived facts re-enter the network)

The second slice of stone 4. Today (4a) a derived fact lands in `production-memory` but never re-enters the
network, so a rule that *consumes* a derived fact never fires. 4b closes the loop: a derived fact becomes a
matchable fact, `fire-rules` iterates to a monotone fixpoint, and a rule chain (A derives X; B fires on X)
fully resolves. No truth-maintenance / no retraction (4c), no `Snapshot` (4d).

## Why

The inserts-only thesis (`project_rete_inserts_only_replay`): the working memory is a pure deterministic
function of `facts × rules`. Forward chaining is "derive a fact → it unlocks the next gate → repeat." 4a built
the single gate; 4b makes the gates chain.

## The fork — grounded: RE-RUN-FROM-SCRATCH, not incremental

**`fire-rules`'s current 4-pass body already recomputes every memory from `facts` each call** — the alpha pass
seeds an empty `PersistentMap` and reads `Session.facts`; root-join/hash-join/production all seed empty and
recompute (`wat/rete.wat:889-933`). So the body is a pure `fn(facts, network, rules) → memories`. That makes
the cascade nearly free:

- **Extract** the current 4-pass body as `fire-once(session) -> session` (recomputes all memories from
  `session.facts`).
- **`fire-rules` becomes a fixpoint driver:** `fire-once` → collect the derived facts from `production-memory`
  → merge them into `facts` (dedup by value-equality) → if `facts` grew, recurse with the enlarged fact set;
  else return the last `fired` session.

Each round re-runs the full match over the enlarged fact set. A derived `ColdAndWindy` added to `facts` is, on
the next round, matched by rule B's alpha exactly like an input fact → B fires. This **is** the pure-replay
thesis, made iterative.

**Rejected (deferred, NAMED): incremental delta-propagation** — splicing new derived facts/tokens into the
existing memories without a full recompute (the real-RETE perf algorithm). Categorically more complex; the perf
path. Deferred exactly like the 3b join-bindings index — correctness-first now, perf when measured.

Grounded deps (all exist): `PersistentMap/values` (`runtime.rs:4342`), `PersistentVector/contains?`
(`collection/eval.rs:1243`, uses `x == item` = structural `Value` equality → works on records), self-recursive
`defn` (`fix-seq`/`fix-text`/`rename-prefix-edits-walk` recurse — wat supports direct recursion).

## What 4b delivers (all WAT — no Rust)

1. **`fire-once`** — the current `fire-rules` 4-pass body, renamed/extracted, `fn(Session) -> Session`.
2. **`collect-derived`** — flatten `production-memory` values (a `PersistentMap` of `PV<Record>`) into one
   `PV<:wat::Record>` (foldl over `PersistentMap/values`, inner foldl `conj`).
3. **`merge-facts`** — fold derived facts into the existing fact `PV`, `conj`-ing only those not already present
   (`PersistentVector/contains?` guard). Dedup by value-equality is the termination guard.
4. **`fire-rules`** (the driver) —
   ```
   fired      = fire-once(session)
   derived    = collect-derived(fired)
   old-facts  = Session/facts(session)
   new-facts  = merge-facts(old-facts, derived)
   (if (= (length new-facts) (length old-facts))
       fired                                        ; fixpoint — no new facts this round
       (fire-rules (Session … new-facts …)))        ; recurse with the enlarged fact set
   ```
   The recursion passes a session with `facts = new-facts`; `fire-once` ignores the incoming memories (it
   recomputes from `facts`), so no need to clear them. The final `fired.facts` is the full closure
   (input + all derived); `production-memory` is the last round's derivations — each exactly once (recomputed
   fresh per round → no cross-round inflation).

## Termination

Monotone-finite (datalog): `facts` grows strictly each round or the loop stops; it is bounded by the finite set
of derivable facts (finite record types × finite field-values drawn from a finite input). The **no-new-facts
round** (length unchanged after dedup) is the guard — no arbitrary round cap (a cap would mask a genuine
user-rule error and pick an arbitrary N). **Known boundary, NAMED, not a 4b bug:** a rule that derives an
unbounded stream of *distinct* facts (e.g. arithmetic in a fact-arg producing `X(n) → X(n+1)`) would not
terminate — a standard datalog property. v1 has no such rule; if one is ever needed, a depth/round safety cap
is its own future stone (let need reveal).

> **✅ THE NEED REVEALED — the back edge this paragraph never had (annotated 2026-09-05).** The cap
> shipped: `fire_fixpoint_delta`'s TERMINATION CAP (`src/rete/kernel/fire/delta.rs`), per-program via
> `(:wat::config::rete::set-max-fire-rounds! n)`. It quotes the sentence above verbatim and answers
> its objection point by point — the measured failure without a cap was
> `memory allocation of 545259536 bytes failed`, naming no rule and no span. The forward edge has
> pointed here since it landed; only the edge back was missing. **Nothing above is retracted**: the
> boundary this stone named is exactly the one that was hit.

## The one contract decision (pinned)

`fire-rules` is a fixpoint over `fire-once`: re-run the full match over a dedup-growing fact set until a round
adds no new fact. Re-run-from-scratch (pure replay), NOT incremental splice. `production-memory` stays the flat
`PV<:wat::Record>` from 4a (the support store is still 4c).

## Files touched

- `wat/rete.wat` — extract `fire-once`; add `collect-derived`, `merge-facts`; rewrite `fire-rules` as the
  fixpoint driver. Update the `fire-rules`/`fire-once` doc comments.
- `tests/probe_arc278_4b_cascade.rs` — the probe (a 2-rule chain).

## Out of scope = REJECTED (not "later")

- **Truth maintenance / retraction / the `{token → [facts]}` support store** — 4c.
- **`:wat::rete::Snapshot`** — 4d.
- **`query` / `collect-rules` / `defrule`** — stone 5 (the probe reads `production-memory` directly).
- **Incremental delta-propagation** — deferred (the perf path; named above).
- **An arbitrary recursion/round cap** — not built (would mask user error; the no-new-facts guard suffices).
- **No Rust change** (4a's `eval-insert` already does RHS construction). No 1a/1b/2/3/4a record or signature
  change beyond renaming the 4-pass body to `fire-once`.
