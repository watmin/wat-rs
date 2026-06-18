# DESIGN — Stone 2b: `insert` + `fire-rules` (the alpha activation slice)

> Arc 278 stone 2, part b. The first runtime slice: stage facts with `insert` (zero activation), then
> `fire-rules` runs them through the network's AlphaNodes via `alpha-match` (2a) and populates **alpha-memory**.
> `fire-rules` is built incrementally — this is its ALPHA slice only (no beta join / no production / no
> cascade yet; those are stones 3/4). Pure WAT engine logic calling the 2a Rust primitive.

## The model (builder, 2026-06-19)
- **Insert phase — WM open, zero activation.** The user stages as many facts as they want; `insert` only
  appends to `Session.facts`. Nothing fires.
- **`fire-rules` locks the WM and (eventually) cascades to fixpoint.** The lock is **structural/free**: pure
  value-semantics — `fire-rules` takes a `Session`, returns a NEW frozen `Session`; the user never holds a
  mid-fire handle. (Stone 4 adds the cascade loop + TM; this slice just runs alpha once.)

## Contract
```
(:wat::rete::insert [session <- :wat::rete::Session  fact <- :wat::Record] -> :wat::rete::Session)
(:wat::rete::fire-rules [session <- :wat::rete::Session] -> :wat::rete::Session)
```
- `insert` — `Session/assoc :facts (PersistentVector/conj (Session/facts session) fact)`. Stage only; return
  the new Session. Pure. Zero activation.
- `fire-rules` (v1 alpha slice) — for each AlphaNode in the network, for each staged fact, run
  `(:wat::rete::alpha-match cond fact)` (cond = the AlphaNode's `tests` head); on `Some(bindings)` store an
  `Element` in alpha-memory; return the new Session with `alpha-memory` populated. NO beta/production/cascade.

## The ONE contract decision — `Element.fact` holds the RECORD (1a refinement)
Change `Element.fact` from `:wat::core::PersistentMap` to `:wat::Record` (1a flagged its field "v1
record-as-map" — tentative; this is that call). The record preserves **type + fields + identity** (query-by-
fact-type, fact-binding `?f <-`, TM-provenance all work on it); it's already EDN-representable (arc 234.7); and
it needs **no conversion** (no `record->map`, no new primitive). Four-questions: Obvious/Simple/Honest/Good-UX
all YES (store the fact as the fact). Update `Element` in `rete.wat` + any constructor site.

## alpha-memory shape (this stone)
`alpha-memory : PersistentMap<node-id (i64), PersistentVector<Element>>` — each AlphaNode → the Elements whose
facts matched it. **Stone 3 may refine** to `node-id → {join-bindings → [Element]}` (the join-index keying,
CLARA-REF §5) — that sub-key is a BETA concern (the join's `binding-keys` aren't known at alpha level), so the
flat shape is the honest alpha-level output, not a build-around.

## Algorithm (`fire-rules` alpha slice — pure WAT)
Thread `alpha-memory` (start empty) through a fold:
- get `network` = `Session/network`, `facts` = `Session/facts`.
- fold the network's node-ids (`PersistentMap/keys`); for each node that IS an AlphaNode
  (`node-kind-label` == "AlphaNode"):
  - `cond` = the AlphaNode's condition = first of `AlphaNode/tests` (a `:wat::WatAST`).
  - fold `facts`: for each `fact`, `(:wat::rete::alpha-match cond fact)`:
    - `Some(bindings)` → `Element` = `(:wat::rete::Element fact bindings)`; append to `alpha-memory[alpha-id]`
      (create the PV if absent).
    - `None` → skip.
- return `Session/assoc :alpha-memory <built map>` (frozen by value-semantics).

All deps exist: `alpha-match` (2a), bare-PV `foldl` (0d.1 — `facts` is bare PV), `node-kind-label` +
`PersistentMap/keys`/`get`/`assoc` + `PersistentVector/conj` + `Record/assoc` (1a/1b). **If a sub-dep turns out
missing, BUILD it as a core primitive — do not hack around it** (builder directive 2026-06-19).

## Proof (FM-2-bis — RED at HEAD)
`tests/probe_arc278_2b_insert_alpha.rs` (RED, un-ignore on green): compile a 1-rule network whose single
condition is `(:user::Temp (?t <- :value) (:wat::core::> ?t 20))`; `insert` `(:user::Temp 25)` (matches) AND
`(:user::Temp 15)` (fails `> 20`); `fire-rules`; then inspect `Session/alpha-memory`:
- exactly **1** AlphaNode is populated (`length (PersistentMap/keys alpha-memory)` == 1),
- that node holds exactly **1** Element (15 was rejected by the constraint — proves activation honors
  `alpha-match` fully, not just type),
- that Element's `bindings` has `"?t"` == `25` (bindings flow from `alpha-match` into the Element).
RED at HEAD: `:wat::rete::insert` / `fire-rules` unknown (compile/Session/Temp/alpha-match all exist).

## Out of scope (affirmative cuts)
- Beta join (Token/Element two-memory split) → stone 3.
- Production firing / RHS inserts / the cascade-to-fixpoint loop / TM → stone 4.
- The `join-bindings` sub-key on alpha-memory → stone 3 (when the join needs the index).
- `retract` (session-level) → its own slice (with TM, stone 4-ish).

## Four questions
- **Obvious?** YES — `insert` stages; `fire-rules` runs facts through alphas → alpha-memory.
- **Simple?** YES — two pure WAT fns; the matcher already exists; a flat alpha-memory; no new Rust.
- **Honest?** YES — zero activation until `fire-rules` (the model); the slice does exactly alpha, no faked beta.
- **Good UX?** YES — value-threaded lifecycle (`compile → insert → fire-rules`); the lock is structural.

## Blast radius
`wat/rete.wat` (`Element.fact` → `:wat::Record`; add `insert` + `fire-rules` + any fold helpers) + the probe.
NO Rust (alpha-match exists) UNLESS a sub-dep is found missing — then build it as a core primitive. No git in
the worker.
