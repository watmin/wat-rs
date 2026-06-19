# BRIEF — Stone P10: annihilate the dead support-chain provenance

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** RUST,
`src/rete/kernel.rs` fire passes only. Build, run the named tests, report verbatim. Another agent weighs.
Behavior-preserving — the differential is the net; do NOT change observable results.

## The work
A token's `matches` (the `(fact, alpha-id)` support chain) is **dead weight in this engine**: it is built,
carried forward by `extend_token`, frozen into the Session, and **never read** for `query`, production firing,
or TM (TM is replay). Yet each token pays `Value::Tuple(Arc::new(vec![fact, i64]))` + a `VectorSync` push to
build it — ~3–4 allocations × join cardinality. **Stop populating it.** Leave `bindings` (the live data) and the
oracle untouched. This dominates the fan-out-40k cell and speeds every workload.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P10-drop-dead-provenance.md` — the grounded weak-point
   (matches has no consumer), the kill, the safety contract, out-of-scope.
2. `src/rete/kernel.rs` — `make_token` (`:313`), `extend_token` (`:547`), the root-join seed sites (batch
   `:489-492`, delta `:1197-1200`), and confirm with your own read that every `token_matches_bindings(...).0`
   (the matches) is either fed to `extend_token` or ignored (`_`) — never consumed for output.
3. `tests/probe_arc278_4c_retraction.rs` — the proof TM is replay (never reads matches); it must stay 4/4.

## The kill (fire passes ONLY)
- **Root-join seeds** (batch `hash`-pass region `:489-492` and delta `:1197-1200`): the token seeded per element
  currently builds `support = Tuple(...)`, `matches_pv = VectorSync::new().push_back(support)`. Replace with an
  **empty** matches PV: `make_token(rpds::VectorSync::new_sync(), bindings)` — drop the support Tuple entirely.
- **`extend_token`** (`:547-567`): currently builds `support = Tuple(Arc::new(vec![el_fact, i64(alpha_id)]))` and
  `new_matches = tok_matches.push_back(support)`. **Drop both** — the extended token carries `tok_matches`
  through unchanged (which is empty). Cleanest: have `extend_token` take only what it still uses (`tok_matches`
  passthrough + `tok_bindings` + `el_bindings`); drop the now-unused `el_fact`/`alpha_id` params and update the
  call sites. (If dropping params ripples awkwardly, keep them and just stop building/pushing the support —
  note which you chose.)
- `matches` STAYS a field on the Token (struct_form[0]) — always an empty PV now. **No type/arity change.**
- **DO NOT TOUCH:** `to_transient`, `to_persistent`, `pm_to_hashmap`, `hashmap_to_pm` (stay lossless), the
  `bindings` handling, the Token/Element record shape, `Value`, or the wat oracle (`wat/rete.wat`).

## STOP triggers
1. Any differential goes red → you changed observable behavior; STOP (matches was supposed to be unread — if
   removing it changed `query`, something DID read it; surface what).
2. `4c_retraction` goes red → TM somehow depended on matches; STOP and report (it must not).
3. A test asserts `matches` *content* (provenance present/shaped) → it checks dead data; STOP, name it, do NOT
   weaken it silently — the orchestrator decides.
4. You reach to change `to_transient`/`to_persistent`, the oracle, `bindings`, or the Token type → out of scope; STOP.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored                  # 4/4 (TM replay, matches-independent)
cargo test --release -p wat --test probe_arc278_deep_cascade -- --include-ignored                    # 2/2 (native==wat==closure)
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored            # 4/4
cargo test --release -p wat --test probe_arc278_P4c_native_retraction                                 # 3/3
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored               # 4/4
for t in 2b_insert_alpha 3a_root_join 3b_hash_join 4a_production_fire 4b_cascade 5a_defrule_query; do cargo test --release -p wat --test probe_arc278_$t -- --include-ignored 2>&1 | grep "test result"; done  # all green
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result            # 1/1
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                                      # green incl round_trip (fires native → empty matches round-trips)
cargo test --release -p wat --lib 2>&1 | grep "test result"                                           # 935/36 (the 36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                            # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                                   # Finished; no NEW warnings
```
Report: the edits to make_token seeds + extend_token (the code); whether you dropped extend_token's
support params or kept-and-ignored; every test result verbatim; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` fire passes (root-join seeds + extend_token). NO oracle, NO to_transient/to_persistent, NO
bindings, NO Token type, NO Value, no new probe. No git.
