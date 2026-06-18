# BRIEF — Stone 1b: `compile` (rule-set → shared network)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A WAT stone (pure wat
added to `wat/rete.wat`). Build, run the named tests, report verbatim. Another agent weighs independently.

## The work
Add `(:wat::rete::compile [rules <- :wat::core::PersistentVector] -> :wat::rete::Session)` to `wat/rete.wat`:
walk each rule's conditions left-to-right and build the network (id→Node `PersistentMap`) with **node
SHARING** (alpha + beta-prefix), via ONE unified find-or-mint dedup. `compile` IS the session constructor:
fresh `Session` with the compiled `network`, empty memories, `facts` empty, `next-id` set. No fire.

## Read FIRST (in order) and implement EXACTLY
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-1b-compile.md` — the algorithm, the condition shape, the
   ONE decision (conditions = raw `form::matches?` clause-forms), the find-or-mint dedup. Implement it.
2. `docs/arc/2026/06/278-rules-engine/CLARA-REFERENCE.md` §4 — the sharing mechanism: reuse a node iff its
   (structure + parent) match; else mint a new id. This is the heart; mirror its logic (not its Clojure).
3. `wat/rete.wat` — the records you build (`AlphaNode`/`RootJoinNode`/`HashJoinNode`/`ProductionNode`,
   `Session`) + `render-dag` (the probe uses it). Build the network as raw node records in `Session.network`
   (v1, as 1a's comment states), keyed by id.
4. `wat/lint.wat` — the ast-walkers: `ast->children`/`ast-name`/`first` (read a condition's head = fact type
   via `(ast-name (first (ast->children cond)))`).
5. `wat/deporder.wat` / `wat/service.wat` — the `PersistentMap` fold/assoc/get + `Vector` fold idioms
   (HashMap idiom works on PersistentMap). You thread the network map + next-id through a fold over rules.
6. `tests/probe_arc278_1b_compile.rs` — remove its `#[ignore]`. It compiles two rules with an identical
   FIRST condition and asserts `render-dag` shows exactly 3 AlphaNodes (the shared one + two divergent) —
   the sharing proof. It is your contract.

## Algorithm (per DESIGN-1b)
Fold over `rules`, threading `(network, next-id)`:
- per rule: `parent := none`; fold its `lhs` conditions left→right:
  - `alpha-id := find-or-mint` an `AlphaNode` whose `tests` == this condition (`:wat::core::=` on the form) → ALPHA sharing.
  - `join-id  := find-or-mint` a `RootJoinNode` (first condition) or `HashJoinNode` (rest) whose
    (condition + `parent`) match → BETA-prefix sharing.
  - link children (alpha→join, prev-parent→join); `parent := join-id`.
  - mint a `ProductionNode(rule-name)` child of the final `parent` (productions are NOT shared).
- `find-or-mint` = scan the network-so-far for a node equal on (kind, structure, parent); reuse its id else
  take `next-id` and increment. Return the final `Session`.

## Engine-source bar (DOGFOOD)
Write `compile` LINT-CLEAN — NO `string::concat` abuse (use `format`/`interpolate`), NO nested-if `=`/`contains?`
ladders (use `contains?`/`cond`). The rete engine's own source is held to its sibling linter's bar. The ONLY
permitted below-bar spot is the EXISTING deliberate fixture in `render-dag` (do NOT touch it). The orchestrator
will dogfood the linter on `wat/rete.wat` in the weigh.

## STOP triggers
1. If the join-key (`binding-keys`) intersection logic is non-trivial — store the condition's `?var`s on the
   join and STOP to report; do not invent join semantics (fire/stone-3 owns join execution).
2. If `:wat::core::=` on the quoted condition forms does NOT give structural equality (sharing won't dedup) —
   STOP, report what equality the forms support.
3. If find-or-mint needs a PersistentMap iteration/scan primitive that doesn't exist — STOP, name it.

## Verify (paste verbatim)
```
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored          # 1/1 GREEN (3 alphas)
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored        # 1/1 (1a still green)
cargo test --release --test test_stdlib_load_order | grep result                          # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                               # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                # 264/1 (or +1 if you add a deftest)
cargo build --release 2>&1 | tail -2                                                       # clean
```
Report: the `compile` + helper source, all outputs verbatim, any STOP hit. Do not claim a green you did not
see. Un-ignore the 1b probe. No git.

## Blast radius
`wat/rete.wat` (add `compile` + find-or-mint helpers) + un-ignore the 1b probe + optionally a `wat-tests/`
deftest. NO Rust. NO change to existing rete records / render-dag (except: leave the fixture). No git.

---

## ⛔ RE-STRIKE (2026-06-18) — SUPERSEDES the above where they conflict

The first 1b attempt was REVERTED in the weigh: it (a) DEFERRED child-edge wiring (left every node's `children`
empty, claiming "stone 3 owns it") — but a network with no edges is not a compiled DAG; and (b) used a
`(foldl … (range 0 n))` + `PersistentVector/get` workaround because the checker rejected `foldl` over a
PersistentVector. Both are now closed/forbidden:

1. **`foldl` over a PersistentVector now TYPE-CHECKS** (stone 0d shipped, 09bdb10b). Use **direct `foldl`**
   over the rule/condition PersistentVectors — NO `range`-index workaround. Same for any map/filter you need.

2. **WIRE THE CHILD EDGES — not optional, not deferred.** `compile` MUST populate each node's `children`:
   `alpha-id.children ∪= join-id`; `prev-parent.children ∪= join-id`; `join.children ∪= production-id`. The
   `children` fields exist on AlphaNode/RootJoinNode/HashJoinNode for exactly this. A leaf ProductionNode has
   no children. This is the heart of "a coherent DAG."

3. **`render-dag` now EMITS EDGES.** Extend it so each line is:
   ```
     <id>  <kind> -> [<child-id> <child-id> ...]
   ```
   children = the node's `children` vector, space-separated inside `[]`; leaves (ProductionNode/QueryNode,
   which have no `children` field) render `-> []`. render-dag must dispatch on kind to read `children` from the
   three node kinds that have it (Alpha/RootJoin/HashJoin) and render `[]` for the leaves.
   ⛔ **PRESERVE THE COMPOUND-CONCAT FIXTURE.** render-dag's line is built with a DELIBERATE nested
   `string::concat` (the `compound-concat-collapse` proof-by-diff target — see the in-source marker). EXTEND
   that nested-concat line to include the ` -> [...]` edge text; KEEP it nested `string::concat`. Do NOT
   collapse it to `format`/`interpolate` — that cleanup is the rete engine's own future job (proof-by-diff),
   not yours. Adding more nested concat is fine; "fixing" it is forbidden.

4. **The probe is STRENGTHENED** (`tests/probe_arc278_1b_compile.rs`, already rewritten — un-ignore BOTH
   tests): `compile_shares_prefix_and_wires_the_chain` asserts the node-kind counts (3 alpha / 1 root-join /
   2 hash-join / 2 production = sharing) AND that the shared RootJoinNode has **2 children** (the divergence —
   proves edges wired); `compile_single_rule_wires_a_connected_chain` asserts a one-rule chain is connected
   (alpha→root-join→production, each with its child). The probe pins the render-dag edge format above. It is
   your contract — make both green WITHOUT editing the probe.

Everything else in the brief above (find-or-mint dedup, the algorithm, STOP triggers, the engine-source bar)
still holds. Verify with the updated probe command:
```
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored          # 2/2 GREEN
```
