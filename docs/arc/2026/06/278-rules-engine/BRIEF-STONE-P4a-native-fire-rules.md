# BRIEF — Stone P4a: native `fire-rules'` (re-run fixpoint cascade)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone (`src/rete/kernel.rs` grows + 1 dispatch arm + 1 TypeScheme). Build, run the named tests, report
verbatim. Another agent weighs.

## The work
Add a native primitive `(:wat::rete::fire-rules' <session>) -> :wat::rete::Session` — the native cascade
fixpoint. It is to the native `fire-once'` what the wat `fire-rules` is to the wat `fire-once`: loop the single
pass, let derived facts re-enter the network, terminate when a round adds no new fact, then return a Session
with `facts = input only`. A **1:1 port of the wat `fire-fixpoint` + `fire-rules`** (`wat/rete.wat:981-1018`).
Re-run-from-scratch each round (NOT delta — that is P4b). It must be **observationally equivalent** to the wat
`fire-rules`: same derived facts (`query(fire-rules' s, T) == query(fire-rules s, T)` for every T), including
the multi-round cascade where a DERIVED fact unlocks a higher rule.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P4-delta-fire.md` — the P4 decomposition; THIS stone is
   P4a (the §"This stone: P4a" section: the algorithm, the reuse/reimplement split, the contract, out-of-scope).
2. `wat/rete.wat:937-1018` — the exact oracle you port: `collect-derived` (`:940-955`), `merge-facts`
   (`:960-972`), `fire-fixpoint` (`:981-998`), `fire-rules` (`:1006-1018`). **Do NOT change the oracle.**
3. `src/rete/kernel.rs` — `eval_fire_once_native` (`:698`, the single pass you call in a loop), the four pass
   fns, `to_transient`/`to_persistent`, the Session struct_form positions (network 0, rules 1, alpha 2, beta 3,
   production 4, facts 5, next-id 6). And `src/runtime.rs` near the `:wat::rete::fire-once'` dispatch arm +
   `src/check.rs` near the `fire-once'` TypeScheme — copy that registration pattern for `fire-rules'`.
4. `tests/probe_arc278_P4a_native_fire_rules.rs` — the differential contract (already live, RED). Do not modify it.

## The algorithm (port of `fire-fixpoint` + `fire-rules`)
1. **Extract a reusable single-pass fn** from `eval_fire_once_native`: `fn fire_once_session(session: &Value,
   sym: &SymbolTable) -> Result<Value, EvalBreak>` = `to_transient` → clear the 3 memories → `alpha_pass` →
   `root_join_pass` → `hash_join_pass` → `production_pass` → `to_persistent`. Then `eval_fire_once_native` =
   eval its AST arg → `fire_once_session(&session, sym)`. **Behavior of `fire-once'` must NOT change** (the P2
   differential is the canary). The fixpoint calls `fire_once_session` directly (NO re-eval of an AST arg per
   round).
2. **`collect_derived(production_pm: &Value) -> Vec<Value>`** — flatten production-memory's per-node
   `PV<Record>` values into one `Vec<Value>` (mirror `collect-derived`).
3. **`merge_facts(facts_pv: &Value, derived: &[Value]) -> Value`** — fold `derived` into the facts
   PersistentVector, conj-ing ONLY facts not already present (structural `==` dedup; mirror `merge-facts`). This
   dedup is the termination guard.
4. **The fixpoint** (Rust `loop` or recursion, mirror `fire-fixpoint`): each round `fired =
   fire_once_session(&cur)`; `derived = collect_derived(fired.production-memory)`; `new_facts =
   merge_facts(cur.facts, &derived)`. If `len(new_facts) == len(cur.facts)` → return `fired`. Else rebuild the
   Session as `fired` but with `facts = new_facts` (so the next round's `fire_once_session` matches input ∪
   derived) and loop. NO arbitrary round cap — termination is the no-new-fact guard, exactly like the oracle
   (monotone-finite / datalog).
5. **`eval_fire_rules_native`** — the dispatch entry: eval the session arg; `let input = session.facts`; run
   the fixpoint → `fired`; rebuild and return `fired` with `facts = input` (mirror `fire-rules` — derived live
   in production-memory, facts holds only the asserted base).

Read the wat helpers' field math against the Session positions; build the new Session `wat__Record` with
`class_fqdn: Arc::new("wat::rete::Session".into())` and `struct_form` in declaration order (see `to_persistent`
for the exact shape).

## Builder directive: build missing deps, never hack around
Everything you need exists (`fire_once_session` after the extraction, `to_transient`/`to_persistent`, `Value`
`==`, the PersistentVector/PersistentMap variants). **If a primitive is genuinely missing → STOP + name it.**

## STOP triggers
1. A needed primitive is missing → STOP, name it.
2. You reach for: delta / incremental / persistent-across-round memories / keying the STORED memory / retract /
   TM / support store / public `fire` / a bench → that is P4b/P4c/P5; STOP. P4a is re-run-from-scratch (loops
   `fire_once_session`).
3. The differential fails and the only fix you see touches the wat ORACLE → STOP (the oracle is the reference;
   `fire-rules'` conforms to it).
4. Extracting `fire_once_session` changes `fire-once'`'s behavior (P2 probe goes red) → STOP; the extraction
   must be behavior-preserving for `fire-once'`.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored   # 4/4 GREEN (native fire-rules' == wat fire-rules; single + cascade)
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored      # 4/4 (fire-once' unchanged by the extraction)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored        # 4/4
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored             # 4/4 (oracle TM intact)
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result    # 1/1
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                              # kernel/matcher unit tests green
cargo test --release -p wat --lib 2>&1 | grep "test result"                                   # 935/36 (the 36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                    # 264/1 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                              # 1/0
cargo build --release 2>&1 | tail -2                                                           # Finished; no NEW warnings
```
Report: the new `fire_once_session` extraction + `collect_derived` + `merge_facts` + the fixpoint +
`eval_fire_rules_native` + the dispatch arm + the TypeScheme; all test outputs verbatim; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` (extract `fire_once_session`; add `collect_derived`, `merge_facts`, the fixpoint,
`eval_fire_rules_native`), `src/runtime.rs` (1 dispatch arm `:wat::rete::fire-rules'`), `src/check.rs` (1
TypeScheme), `tests/probe_arc278_P4a_native_fire_rules.rs` (already live). NO change to the four passes, the
keyed join, the `WorkingMemory` shape, or the oracle. No git.
