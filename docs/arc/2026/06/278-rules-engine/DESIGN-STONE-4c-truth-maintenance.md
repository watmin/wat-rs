# DESIGN — Stone 4c: truth maintenance / retraction (on the wat oracle)

The third slice of stone 4. A `retract`ed fact must remove not just itself but **every derived fact whose
support depended on it**, transitively. On the re-run-from-scratch oracle this falls out of replay — *once the
fact model is corrected*. No `Snapshot` (4d), no Rust kernel (the perf arc).

## Why — and the bug grounding revealed

The inserts-only/pure-replay thesis (`project_rete_inserts_only_replay`) promises TM for free: WM is a pure
function of `facts × rules`, so removing a fact and recomputing drops its consequences. But grounding the
current engine found that **4b's `fire-rules` conflates input and derived facts**, breaking exactly this:

```
fire-rules recurses with `facts = new-facts` (= input ∪ derived, merged by merge-facts),
and the final returned Session has `facts` = the whole closure (input + ALL derived).
```

So a `retract` of an input fact followed by a re-fire would re-derive from a fact set that **still contains the
derived facts** → the consequence never vanishes → TM broken. It also violates `DESIGN.md:288`: *"Input facts
are kept distinct from the final WM precisely so the run is replayable — provenance, not just a snapshot."*

4c fixes this: **`Session.facts` = asserted/input facts only** (the retractable base); the derived closure
lives in `production-memory` (where 4a/4b already put it). The fixpoint still accumulates derived facts into a
*working* set for matching (so cascades fire), but that working set is internal — it never becomes the
returned `Session.facts`.

## What 4c delivers (all WAT — no Rust)

1. **Fact-model fix** — split the fixpoint from the result:
   - Rename the current recursive driver `fire-rules` → **`fire-fixpoint`** (unchanged body: it accumulates
     derived facts into `facts` across rounds for matching, returns the fully-propagated session).
   - New **`fire-rules`** wraps it: capture the original input, run `fire-fixpoint`, then reconstruct the
     returned Session with **`facts` = the original input** (memories + production-memory from the fixpoint
     result):
     ```
     (defn fire-rules [session]
       (let [input (Session/facts session)
             fired (fire-fixpoint session)]
         (Session (network fired) (rules fired) (alpha-memory fired) (beta-memory fired)
                  (production-memory fired) input (next-id fired))))
     ```
   - Net effect: matching still sees input ∪ derived (cascades work — 4b stays green); the *retractable base*
     is input only.
2. **`retract` verb** — `(:wat::rete::retract <session> <fact>) -> <session>`: remove `fact` (by value
     equality) from `Session.facts`, reconstruct the Session (zero activation — symmetric with `insert`, which
     also only stages; `fire-rules` does the recompute). Remove via `foldl` + `contains?`-style guard (conj all
     `f` where `(not (= f fact))`), mirroring `merge-facts`.
3. **TM falls out:** `assert → fire → derived; retract a support → fire → derived gone` (transitively — a
     derived fact that fed another derived fact takes the whole chain down, because nothing re-derives the
     root).

## The one contract decision (pinned)

`fire-rules` returns a Session whose `facts` = the **asserted input only**; the derived closure is in
`production-memory`. `retract` is the inverse of `insert` (stages a removal-by-value; `fire-rules` recomputes).
TM = pure replay over the corrected fact model — no support-store/`matches`-chain cascade needed on the oracle
(that machinery is the Rust kernel's path; here, recompute-from-scratch *is* the cascade).

## Files touched

- `wat/rete.wat` — rename `fire-rules`→`fire-fixpoint`; add the wrapping `fire-rules`; add `retract`; comments.
- `tests/probe_arc278_4c_retraction.rs` — the probe.

## Out of scope = REJECTED (not "later")

- **`:wat::rete::Snapshot`** state blob — 4d.
- **The Rust fire kernel / support-store + `matches`-chain cascade** — the perf arc
  (`PERF-ARC-rust-fire-kernel.md`); on the oracle, replay subsumes it.
- **`query` / `defrule` / `collect-rules`** — stone 5 (the probe reads `production-memory` directly).
- **No Rust change. No record/signature change** beyond the `fire-rules`/`fire-fixpoint` split + the new
  `retract` verb.
