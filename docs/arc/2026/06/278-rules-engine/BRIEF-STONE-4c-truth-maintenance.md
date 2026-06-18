# BRIEF — Stone 4c: truth maintenance / retraction (on the wat oracle)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A PURE
WAT stone (`wat/rete.wat` only — NO Rust). Build, run the named tests, report verbatim. Another agent weighs.

## The work
Make `retract` drop a fact and, transitively, every derived fact whose support depended on it. On the
re-run-from-scratch oracle this is pure replay — but ONLY after fixing a fact-model bug grounding found:
**4b's `fire-rules` returns `Session.facts` = the whole closure (input + derived)**, so retract-then-refire
would re-derive from a fact set that still contains the consequences. Two parts: (1) keep input distinct from
derived; (2) add the `retract` verb. NO `Snapshot` (4d), NO Rust kernel (the perf arc).

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-4c-truth-maintenance.md` — the bug, the fact-model fix
   (`fire-rules`/`fire-fixpoint` split), the `retract` verb, the pinned contract, out-of-scope.
2. `wat/rete.wat` — the CURRENT `fire-rules` (the recursive fixpoint driver you rename to `fire-fixpoint`);
   `insert` (`:471-482`, the stage-only / Session-reconstruct idiom `retract` mirrors); the `Session` record
   (`:124-131`) + its 7-field constructor order (network, rules, alpha-memory, beta-memory, production-memory,
   facts, next-id); `merge-facts` (the `foldl` + `contains?` idiom `retract`'s removal mirrors).
3. `tests/probe_arc278_4c_retraction.rs` — the contract (already live, RED). Do not modify it.

## Part 1 — the fact-model fix (`fire-rules` / `fire-fixpoint` split)
- **Rename** the current recursive `fire-rules` → **`fire-fixpoint`** (body UNCHANGED — it still accumulates
  derived facts into `facts` across rounds so cascades match; update the recursive self-call to
  `fire-fixpoint`). It returns the fully-propagated session (whose `facts` = the closure — that's fine, it's
  internal now).
- **Add a new `fire-rules`** that wraps it and restores `facts` = the original input:
  ```
  (:wat::core::defn :wat::rete::fire-rules
    [session <- :wat::rete::Session]
    -> :wat::rete::Session
    (:wat::core::let [input (:wat::rete::Session/facts session)
                      fired (:wat::rete::fire-fixpoint session)]
      (:wat::rete::Session
        (:wat::rete::Session/network           fired)
        (:wat::rete::Session/rules             fired)
        (:wat::rete::Session/alpha-memory      fired)
        (:wat::rete::Session/beta-memory       fired)
        (:wat::rete::Session/production-memory fired)
        input                                              ;; <- KEY: input only, NOT the closure
        (:wat::rete::Session/next-id           fired))))
  ```
  Matching still sees input ∪ derived (4b cascade stays green); the retractable base is input only.

## Part 2 — the `retract` verb
- `(:wat::core::defn :wat::rete::retract [session <- :wat::rete::Session  fact <- :wat::Record] -> :wat::rete::Session ...)`:
  remove `fact` (by value equality) from `Session.facts`, reconstruct the Session (all other fields passed
  through — zero activation, symmetric with `insert`; `fire-rules` does the recompute). Build `facts'` with a
  `foldl` over `Session.facts`, `conj`-ing each `f` where `(:wat::core::not (:wat::core::= f fact))` (mirror
  `merge-facts`; `=` is structural on records). Reconstruct via the 7-field `Session` constructor.

## Builder directive: build missing deps, never hack around
Deps SHOULD all exist (`foldl`, `=`, `not`, `PersistentVector`/`conj`, the Session accessors + constructor).
**If a core primitive is genuinely missing → STOP + name it.**

## Engine-source bar (DOGFOOD)
LINT-CLEAN — `cond`/`contains?` over nested `if`; `format`/`interpolate` over nested `concat`. The ONLY
below-bar spot is the EXISTING `render-dag` compound-concat FIXTURE — do NOT touch it.

## STOP triggers
1. A needed core primitive is missing → STOP, name it.
2. You reach for a support-store / `matches`-chain cascade / delta retraction → that's the Rust-kernel perf
   arc; on the oracle, recompute-from-scratch IS the cascade. STOP if tempted.
3. You reach for `Snapshot` / `query` / `defrule` → 4d / stone 5; STOP.
4. The `fire-fixpoint` body needs a behavior change (beyond the rename + self-call rename) → STOP and
   describe; it should be a verbatim rename.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored        # 4/4 GREEN
cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored            # 4/4 (cascade still green)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored      # 4/4
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored            # 4/4
cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored             # 3/3
cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored           # 3/3
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored             # 3/3
cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored              # 1/1
cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored                 # 2/2
cargo test --release --test test_stdlib_load_order | grep result                               # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                    # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                     # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                            # Finished; 25 warnings (NO new — pure WAT)
```
Report: `fire-fixpoint` (renamed) + the new `fire-rules` + `retract` source verbatim; all outputs verbatim;
any STOP hit. No git.

## Blast radius
`wat/rete.wat` ONLY (rename `fire-rules`→`fire-fixpoint`; new wrapping `fire-rules`; new `retract`; comments) +
`tests/probe_arc278_4c_retraction.rs` (already live). NO Rust. NO record/signature change. No git.
