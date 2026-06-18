# BRIEF — Stone 4b: cascade-to-fixpoint (derived facts re-enter the network)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE
WAT stone (`wat/rete.wat` only — NO Rust). Build, run the named tests, report verbatim. Another agent weighs.

## The work
Make `fire-rules` iterate to a fixpoint so a derived fact re-enters the network and a rule that consumes it
fires. Today (4a) `fire-rules` is single-pass: a derived `ColdAndWindy` lands in `production-memory` but never
becomes a matchable fact, so a downstream rule never sees it. Re-run the full match over a dedup-growing fact
set until a round adds no new fact. NO truth-maintenance / retraction (4c), NO `Snapshot` (4d).

## The approach — RE-RUN-FROM-SCRATCH (grounded; do NOT do incremental)
`fire-rules`'s current 4-pass body ALREADY recomputes every memory from `Session.facts` each call (every pass
seeds an empty `PersistentMap`). So the cascade is: extract that body as `fire-once`, then loop it — collecting
derived facts, merging them into `facts` (dedup), and recursing until `facts` stops growing. Do NOT attempt
incremental delta-propagation (splicing into existing memories) — that is the deferred perf path; STOP if you
find yourself reaching for it.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-4b-cascade-fixpoint.md` — the full contract: the fork
   decision (re-run-from-scratch), the fixpoint shape, the termination argument, out-of-scope.
2. `wat/rete.wat` — the CURRENT `fire-rules` (`:885-933`, the 4-pass body you extract as `fire-once`); the
   `Session` record (`:124-131`) + its constructor argument order (network, rules, alpha-memory, beta-memory,
   production-memory, facts, next-id); `insert` (`:471-482`, the Session-reconstruct idiom); `append-token`
   (`:556`, the Some/None match idiom). For a self-recursive `defn` reference (the fixpoint driver calls
   itself): `wat/fix.wat` `fix-seq` / `fix-text-seq-edits` recurse directly — wat supports `defn` self-call.
3. `tests/probe_arc278_4b_cascade.rs` — the 2-rule-chain contract (already live, RED). Do not modify it.

## The structure to build (all in `wat/rete.wat`)
1. **`fire-once`** — rename/extract the CURRENT `fire-rules` body verbatim: `[session <- :wat::rete::Session]
   -> :wat::rete::Session`, the alpha → root-join → hash-join → production passes, reconstructing the Session.
   No behavior change — it is exactly today's `fire-rules`, just renamed.
2. **`collect-derived`** — `[prod-mem <- :wat::core::PersistentMap] -> :wat::core::PersistentVector`: flatten
   `production-memory`'s values into one `PV<:wat::Record>`. Outer foldl over `(:wat::core::PersistentMap/values
   prod-mem)` (each value is a `PV<:wat::Record>`), inner foldl `(:wat::core::PersistentVector/conj …)`.
3. **`merge-facts`** — `[facts <- :wat::core::PersistentVector  derived <- :wat::core::PersistentVector] ->
   :wat::core::PersistentVector`: foldl over `derived`, `conj` a fact into `facts` ONLY if
   `(:wat::core::PersistentVector/contains? facts f)` is false (dedup by value-equality — the termination guard).
4. **`fire-rules`** (rewrite as the fixpoint driver) — `[session <- :wat::rete::Session] -> :wat::rete::Session`:
   ```
   (let [fired     (fire-once session)
         derived   (collect-derived (Session/production-memory fired))
         old-facts (Session/facts session)
         new-facts (merge-facts old-facts derived)]
     (if (= (length new-facts) (length old-facts))
       fired                                                   ; fixpoint — no new fact this round
       (fire-rules (Session network rules amem bmem pmem new-facts next-id))))  ; recurse, facts enlarged
   ```
   For the recursion, reconstruct a Session from `fired` (or `session`) with `facts = new-facts`. `fire-once`
   ignores the incoming memories (it recomputes from `facts`), so any memory slot is fine to pass through;
   keep `network`/`rules`/`next-id` from the session. Self-recursion in `defn` is supported.
5. Update the doc comments: `fire-once` (the single-pass cycle) + `fire-rules` (the fixpoint over `fire-once`;
   note re-run-from-scratch, termination by no-new-facts).

## Builder directive: build missing deps, never hack around
Deps SHOULD all exist (`PersistentMap/values`, `PersistentVector/contains?`/`conj`, `length`, `foldl`, `=`,
the Session accessors + constructor, `if`). **If a core primitive is genuinely missing → STOP + name it.**

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `format`/`interpolate` over nested `concat`; `cond`/`contains?` over nested `if`. The ONLY
below-bar spot is the EXISTING `render-dag` compound-concat FIXTURE — do NOT touch it.

## STOP triggers
1. A needed core primitive is missing → STOP, name it (do NOT improvise).
2. You reach for incremental delta-propagation (splicing derived facts/tokens into existing memories instead of
   re-running `fire-once`) → that is the deferred perf path; STOP.
3. You reach for truth-maintenance / retraction / the `{token → [facts]}` support store / `Snapshot` / `query`
   / `defrule` → that is 4c/4d/stone-5; STOP.
4. `fire-once` (the extracted body) needs a behavior change to work in the loop → STOP and describe; it should
   be a verbatim extraction.
5. The fixpoint does not terminate on the probe (hangs) → STOP; the dedup guard (contains? before conj) is what
   makes `merge-facts` bounded — check it before assuming a deeper problem.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored        # 4/4 GREEN
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored  # 4/4 (single-rule still green)
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored         # 4/4
cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored          # 3/3
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored        # 3/3
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored          # 3/3
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored           # 1/1
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored              # 2/2
cargo test --release --test test_stdlib_load_order | grep result                            # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                 # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                  # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                         # Finished; no NEW warnings (25)
```
Report: the `fire-once`/`collect-derived`/`merge-facts`/`fire-rules` source verbatim; all outputs verbatim; any
STOP hit. No git.

## Blast radius
`wat/rete.wat` ONLY (`fire-rules` → `fire-once` + the fixpoint driver + 2 helpers + comments) +
`tests/probe_arc278_4b_cascade.rs` (already live). NO Rust. NO record/signature change. No git.
