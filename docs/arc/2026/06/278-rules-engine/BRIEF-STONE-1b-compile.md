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
