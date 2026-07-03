# BRIEF — Stone 7strat-native: STRATIFIED negation in the native kernel

**The work (one paragraph).** The native fire path (`fire-rules` → `fire-rules'` → `eval_fire_rules_native`)
is raw single-fixpoint and gives the wrong answer on negation-over-derived: `Ok=2` where the wat oracle
`fire-rules-spec` (and Clara) give `Ok=1`. Port the oracle's stratification **natively in Rust** as a
self-contained parallel impl, so `fire-rules` matches the oracle. This is a **line-for-line port** of the
proven wat oracle — same algorithm, in Rust. The RED probe
`tests/rete/probe_arc278_7strat_native_differential.rs::differential_stratified_negation` is the spec: make
it green (native == oracle == `{Bad:1, Ok:1}`).

**Read in order (the rooms):**
1. `wat/rete.wat:1543-1800` — **the algorithm you are porting.** `rule-produces` (RHS insert types),
   `rule-negates` (LHS `:not` types), `stratify-sweep`, `stratify-fix` (negation cycle → raise),
   `rule-stratum`, `stratify`, `fire-stratified-loop`, `fire-stratified`. Mirror each in Rust.
2. `wat/rete.wat:52-55` — the `Rule` record shape: `{name:String, lhs:PV<WatAST>, rhs:PV<WatAST>}`. Native
   reads `lhs`/`rhs` as Values; the type FQDN is `ast-name` of the first child of the fact form, colon-stripped
   (see `rule-produces` at 1546-1565 for the exact walk).
3. `src/rete/kernel.rs:1977-1997` — `eval_fire_rules_native`, the dispatch point. Today: evaluate the session
   arg, call `fire_fixpoint_delta(&session, sym, None)`. You add the stratify+dispatch here.
4. `src/rete/kernel.rs:1384` — `fire_fixpoint_delta(session, sym, support)` — the per-stratum engine you call.
5. `src/rete/kernel.rs:1047` — `collect_derived(production_pm) -> Vec<Value>` — reuse for per-stratum derived.
6. `src/rete/kernel.rs:1320-1360` — **the proven native→wat call pattern**: build a `WatAST::List` and
   `crate::runtime::eval_inner(&call, &env, sym)`. Use this to invoke the pure-wat `:wat::rete::compile` on a
   per-stratum rule subset (there is no native compiler; compile is the shared front-end).

**Implementation sketch (fill it in; do not invent a different shape):**
```
eval_fire_rules_native(session):
  let rules  = <read Session/rules field of the session Value>
  let strata = native_stratify(&rules, sym)          // port stratify: produces/negates/sweep/fix → HashMap<String,i64>; cycle → Err
  let max_s  = <max rule_stratum over rules, 0 if empty>
  if max_s == 0 { return fire_fixpoint_delta(&session, sym, None) }   // UNCHANGED fast path
  // else — port fire-stratified-loop, bottom→top:
  let mut acc_facts = <Session/facts>;  let mut acc_derived = vec![];
  for s in 0..=max_s {
      let stratum_rules = <rules filtered to rule_stratum == s>;
      let sub_sess = <invoke wat compile on stratum_rules via the kernel.rs:1345 pattern>;
      let sub_sess = <seed sub_sess with each fact in acc_facts via wat insert (same native→wat pattern)>;
      let fired = fire_fixpoint_delta(&sub_sess, sym, None)?;
      acc_derived.extend(collect_derived(<fired production_memory>));
      acc_facts = <fired Session/facts>;
  }
  <return a Session Value: production_memory = {0: acc_derived}, facts = input>   // mirror fire-stratified + fire-rules-spec exactly
```

**Blast radius:** `src/rete/kernel.rs` ONLY. New: `native_stratify` + its helpers + `fire_rules_stratified` +
the dispatch in `eval_fire_rules_native`. No new public verbs.

**STOP triggers (rejection criteria — ship nothing, report):**
- **STOP-1:** if you cannot invoke the wat `:wat::rete::compile` verb from Rust per stratum via the
  `kernel.rs:1320-1360` pattern, STOP and report. Do **not** work around it by gating node activation inside
  `fire_fixpoint_delta` — that is a different (rejected) design.
- **STOP-2:** if the session Value's field layout (`Session/rules`, `Session/facts`, `production-memory`) is
  not readable the way `fire_fixpoint_delta` already reads it, STOP and report — do not guess a layout.
- **Do NOT edit `wat/rete.wat`** — the oracle is the reference; it does not change.
- **Do NOT add a `native?`/mode flag** anywhere. The wat oracle and native kernel are SEPARATE parallel impls.
- **Do NOT touch `src/rete/matcher.rs`** — rete stays pure; no negation rewrite, no RHS eval.

**Prior pattern to mirror:** the native↔oracle differential shape in
`tests/rete/probe_arc278_8custom_native_differential.rs` (native == oracle assertion) and the delta-fix in
`fire_fixpoint_delta` (commit `1cf61bdb`) show the native kernel's conventions.

**Done = the gate is green:**
- `cargo test --release -p wat --test rete probe_arc278_7strat_native_differential` → 2/2 pass
  (native `(1,1)` == oracle `(1,1)`).
- `cargo test --release -p wat --test rete` → whole rete suite green (no regression to P4a/deep-cascade/P6/8custom).
- Report the neg-case counts you observed (native + oracle) so the orchestrator can weigh vs Clara's `neg.clj`.
