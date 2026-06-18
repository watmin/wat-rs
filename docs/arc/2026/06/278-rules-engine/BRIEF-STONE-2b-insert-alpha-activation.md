# BRIEF — Stone 2b: `insert` + `fire-rules` (alpha activation slice)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE WAT stone
(`wat/rete.wat`) + one 1a record refinement. Build, run the named tests, report verbatim. Another agent weighs.

## The work
1. **Refine `Element`**: change its `fact` field from `:wat::core::PersistentMap` to `:wat::Record` (store the
   fact record directly — type-preserving, no conversion; 1a flagged it "v1 record-as-map"). Update EVERY
   `Element` construction site so it passes a record (grep `:wat::rete::Element ` across `wat/` + `tests/`).
2. **`(:wat::rete::insert [session <- :wat::rete::Session  fact <- :wat::Record] -> :wat::rete::Session)`** —
   stage only: `Session/assoc :facts (PersistentVector/conj (Session/facts session) fact)`. ZERO activation.
3. **`(:wat::rete::fire-rules [session <- :wat::rete::Session] -> :wat::rete::Session)`** — the ALPHA SLICE:
   run each staged fact through every AlphaNode via `alpha-match`; store matching Elements in alpha-memory;
   return the new Session. NO beta join / NO production / NO cascade (stones 3/4).

## Read FIRST (in order) and implement EXACTLY
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-2b-insert-alpha-activation.md` — the model (zero activation
   until fire-rules), the contract, the `Element.fact`→record decision, the alpha-memory shape, the algorithm.
2. `wat/rete.wat` — the `Element`/`AlphaNode`/`Session` records; `node-kind-label` (:137) to filter AlphaNodes;
   `render-dag` (:155) for the network-iteration idiom (`PersistentMap/keys` + `foldl` + `PersistentMap/get`);
   `network-add-child` (:~250) for the `:wat::Record/assoc` field-update idiom. `compile` (find-or-mint) shows
   `AlphaNode` carries `tests` = `(PersistentVector cond)` — the condition is `(first tests)` ... actually
   `(:wat::core::get tests 0)` → the `:wat::WatAST` cond.
3. `src/rete/matcher.rs` (stone 2a) — `(:wat::rete::alpha-match cond fact) -> Option<PersistentMap>`. Call it
   per (alpha, fact); `Some(bindings)` → make `(:wat::rete::Element fact bindings)`.
4. `tests/probe_arc278_2b_insert_alpha.rs` — remove the 3 `#[ignore]`s. It is your contract.

## Algorithm (`fire-rules` alpha slice — pure WAT, thread alpha-memory)
- `network = (Session/network session)`, `facts = (Session/facts session)`.
- fold `(PersistentMap/keys network)`; for each `node-id` whose node `(node-kind-label node) == "AlphaNode"`:
  - `cond = (get (AlphaNode/tests node) 0)`  (the `:wat::WatAST` condition; un-`Option` it).
  - fold `facts`: `(:wat::rete::alpha-match cond fact)` → `Some(bindings)` ⇒ append
    `(:wat::rete::Element fact bindings)` to the PV at `alpha-memory[node-id]` (create if absent); `None` ⇒ skip.
- return `(Session/assoc :alpha-memory <built-map>)`. `insert` is the trivial stage-conj.

## Builder directive (2026-06-19): BUILD missing deps, do NOT hack around
All deps SHOULD exist: `alpha-match` (2a), bare-PV `foldl` (0d.1 — `facts` is a bare PV), `node-kind-label`,
`PersistentMap/keys`/`get`/`assoc`, `PersistentVector/conj`, `Record/assoc`. **If any needed core primitive is
genuinely missing, STOP and report it (name it) — do NOT improvise a workaround.** The orchestrator will build
the dep as its own stone. (record→map is NOT needed — we store the record.)

## Engine-source bar (DOGFOOD)
Write `insert`/`fire-rules` LINT-CLEAN — `format`/`interpolate` for any strings, `cond`/`contains?` not nested-`if`
ladders. The ONLY permitted below-bar spot is the EXISTING `render-dag` compound-concat fixture — do NOT touch it.

## STOP triggers (HALT + report; do NOT improvise)
1. A needed core primitive is missing (per the directive) → STOP, name it.
2. Changing `Element.fact` breaks a consumer you cannot cleanly update → STOP, report which.
3. The alpha-memory needs a `join-bindings` sub-key to satisfy the probe → it does NOT (flat `node-id → [Element]`
   for v1); if you think it does, STOP and re-read the DESIGN.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored        # 3/3 GREEN
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored           # 1/1 (Element changed — still green)
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored              # 2/2 (compile still green)
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored           # 3/3 (matcher still green)
cargo test --release --test test_stdlib_load_order | grep result                             # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                  # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                   # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                          # clean
```
Report: the `Element` change + `insert`/`fire-rules` source + any helper, all outputs verbatim, any STOP hit,
and any Element construction sites you updated. Un-ignore the 3 probe tests. No git.

## Blast radius
`wat/rete.wat` (`Element.fact` → `:wat::Record`; add `insert` + `fire-rules` + fold helpers) + the probe
(un-ignore) + any Element construction site. NO Rust (alpha-match exists) unless a sub-dep is missing → STOP. No git.
