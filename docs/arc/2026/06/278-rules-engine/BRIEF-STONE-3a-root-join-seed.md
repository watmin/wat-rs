# BRIEF — Stone 3a: `RootJoinNode` seeding (first beta slice)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE WAT stone
(`wat/rete.wat`) — grow `fire-rules` + one 1a record refinement. Build, run named tests, report verbatim.

## The work
1. **Refine `Token`**: change `matches` from bare `:wat::core::PersistentVector` to
   `:wat::core::PersistentVector<(wat::Record,wat::core::i64)>` — a PV of `(fact, node-id)` **tuples** (the
   support entry is a heterogeneous positional pair; a tuple types it honestly, a vec cannot). Update any Token
   construction site so it builds the tuple form.
2. **Grow `fire-rules` with a root-join pass** (runs AFTER the existing alpha pass): for each AlphaNode that has
   Elements, follow its `children`; for each child that is a `RootJoinNode`, seed one Token per Element into
   beta-memory. NO hash-join (3b), NO production (4).

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-3a-root-join-seed.md` — the contract, the tuple decision,
   the algorithm, the flat beta-memory shape, the out-of-scope cuts.
2. `wat/rete.wat` — the `Token`/`RootJoinNode`/`AlphaNode`/`Element`/`Session` records; the EXISTING
   `fire-rules` + `activate-alpha`/`activate-fact` (2b) you extend; `node-kind-label`; how `AlphaNode/children`
   holds child node-ids (1b wired alpha→join). `Element/fact` + `Element/bindings` accessors (2b).
3. `docs/arc/2026/06/278-rules-engine/CLARA-REFERENCE.md` §2 — Token born at RootJoinNode/right-activate as
   `(->Token [[fact node-id]] alpha-bindings)`; we use a tuple for the entry.
4. `tests/probe_arc278_3a_root_join.rs` — remove the 3 `#[ignore]`s. It is your contract.

## Algorithm (extends `fire-rules`, pure WAT)
1. alpha pass (2b) → alpha-memory (unchanged).
2. root-join pass — fold the AlphaNode ids; for each with `els = alpha-memory[alpha-id]`:
   - for each `child-id` in `(AlphaNode/children node)` where the child node `(node-kind-label child) == "RootJoinNode"`:
     - fold `els`: per `el`, `Token = (:wat::rete::Token (:wat::core::PersistentVector (:wat::core::Tuple (:wat::rete::Element/fact el) alpha-id)) (:wat::rete::Element/bindings el))`; append to `beta-memory[child-id]` (create PV if absent).
3. return Session with both alpha-memory (step 1) AND beta-memory (step 2) set; production-memory unchanged.

## Builder directive (2026-06-19): build missing deps, do NOT hack around
Deps SHOULD all exist (Tuple ctor, Element/Token accessors, AlphaNode/children, node-kind-label, PersistentMap/
PersistentVector ops, bare-PV foldl). If a needed core primitive is genuinely missing → **STOP and name it**;
do NOT improvise. (The orchestrator builds the dep.)

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `format`/`interpolate`, `cond`/`contains?` not nested-`if`. The ONLY below-bar spot is the EXISTING
`render-dag` compound-concat fixture — do NOT touch it.

## STOP triggers
1. A needed core primitive is missing → STOP, name it.
2. Changing `Token.matches` breaks a construction site you can't cleanly update → STOP, report which.
3. You find yourself needing a `join-bindings` sub-key or a HashJoinNode cross → that's stone 3b; STOP (3a is
   root-join SEEDING only).

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored      # 3/3 GREEN
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored    # 3/3 (alpha pass still green)
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored       # 1/1 (Token changed — still green)
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored          # 2/2
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored        # 3/3
cargo test --release --test test_stdlib_load_order | grep result                         # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                              # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                               # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                      # clean
```
Report: the `Token` change + the `fire-rules` root-join pass source + helpers, all outputs verbatim, any STOP
hit, any Token construction sites updated. Un-ignore the 3 probe tests. No git.

## Blast radius
`wat/rete.wat` (`Token.matches` → PV of tuples; `fire-rules` root-join pass + helpers) + the probe (un-ignore)
+ any Token construction site. NO Rust unless a sub-dep is missing → STOP. No git.
