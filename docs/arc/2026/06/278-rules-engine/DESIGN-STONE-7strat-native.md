# DESIGN — Stone 7-strat-native: STRATIFIED negation in the NATIVE kernel

**Status:** STRIKE-READY. Probe RED (`tests/rete/probe_arc278_7strat_native_differential.rs`: native `(1,2)` ≠ oracle `(1,1)`).
**Depends on:** the wat ORACLE stratification (`fire-stratified`, built `bb6fb0f9`) as the reference; the native
delta engine (`fire_fixpoint_delta`, P4b); the native base-negation filter (7-b). This is the banked `7-strat`
from `DESIGN-STONE-7-negation.md:79-84`, on the native side.

## Why

The wat oracle (`fire-rules-spec` → `fire-stratified`) already orders strata so a `:not` over a DERIVED fact fires
only after its producer stratum closes. The native fast path (`fire-rules` → `fire-rules'` →
`eval_fire_rules_native`) is still raw `fire_fixpoint_delta` — one fixpoint, no stratification — so
negation-over-derived fires a round early and leaks an extra derivation. `neg.wat` (A(1),A(2); `mark-bad` derives
Bad(2); `ok` = A with no Bad): correct `{Bad:1, Ok:1}`; native leaks `Ok=2`.

The lockstep doctrine (`PARI GRADV, VNA VERITAS`; memory `feedback_wat_oracle_rust_ui_lockstep_no_flag`): the wat
exprs are the ORACLE (correctness), the native kernel is the UI (the fast path). They are **two parallel,
self-contained impls that move in lockstep** — NOT one function with a `native?` flag (the reverted category
error). So native gets its OWN stratification, ported from the oracle, differential-tested against it.

## The algorithm — a faithful port of the oracle, in Rust

Mirror `fire-stratified` + `fire-stratified-loop` (`wat/rete.wat:1704-1800`) natively:

```
eval_fire_rules_native(session):
  rules  = Session/rules
  strata = native_stratify(rules)                 # PORT: rule_produces/rule_negates/sweep/fix/rule_stratum
  max_s  = max rule_stratum over rules            # (0 if no negation-over-derived)
  if max_s == 0:
      return fire_fixpoint_delta(session)          # UNCHANGED fast path — 99% of rule sets, byte-identical to today
  # else — stratified drive (PORT of fire-stratified-loop), bottom→top:
  acc_facts   = Session/facts
  acc_derived = []
  for s in 0..=max_s:
      stratum_rules = rules.filter(|r| rule_stratum(r, strata) == s)
      sub_sess = compile(stratum_rules)            # the SHARED wat compiler, invoked from Rust (kernel.rs:1345 pattern)
      sub_sess = seed sub_sess with acc_facts      # so this stratum's :not sees the complete prior strata
      fired    = fire_fixpoint_delta(sub_sess)
      acc_derived = merge(acc_derived, collect_derived(fired.production_memory))   # collect_derived native, kernel.rs:1047
      acc_facts   = fired.facts
  return session-shaped result: production_memory = {0: acc_derived}, facts = input   # mirror fire-stratified + fire-rules-spec
```

**Why per-stratum fresh compile (mirror the oracle exactly):** the oracle recompiles each stratum into a fresh
sub-network to eliminate the shared-alpha duplicate-edge doubling AND to enforce the ordering. The native mirror
does the identical thing so the differential is a line-for-line correspondence, not a re-derivation. `compile` is
the ONE shared front-end (both fire-paths consume wat-compiled sessions today); the differential tests *fire +
stratify*, not compile. Stratification LOGIC lives twice (wat + Rust); the compiler is shared, as it already is.

## The one contract decision

`eval_fire_rules_native` stays the single native entry (`fire-rules'`). It DISPATCHES on `max_stratum`:
`0` → today's `fire_fixpoint_delta` (unchanged, zero perf cost for non-stratified rule sets); `>0` → the native
stratified drive. No new public verb, no flag anywhere in wat. `negation cycle` → the same `Err` the oracle raises.

## Out of scope = rejected (named)

- **Perf of per-stratum recompile.** The recompile cost hits ONLY rule sets with stratified negation. It is
  measured in NEXT-2 (the stress matrix under load). If it proves hot, in-network stratum-gating (no recompile)
  becomes a named follow-up THEN — not now. Correctness-first; the faithful port is the low-risk first strike.
- **Touching the wat oracle** (`fire-stratified*`, `stratify*`) — it is the reference; it does not change.
- **`src/rete/matcher.rs`** — rete stays pure; no RHS eval, no engine-internal negation rewrite.

## Files

- `src/rete/kernel/` — `native_stratify` (`kernel/stratify.rs`) + helpers (port of `rule-produces`/`rule-negates`/`stratify-sweep`/
  `stratify-fix`/`rule-stratum`/`stratify`, reading `Rule.lhs`/`Rule.rhs` AST Values) + `fire_rules_stratified`
  (port of `fire-stratified-loop`, per-stratum compile via the native-call pattern) + the
  `max_stratum` dispatch in `eval_fire_rules_native` (both `kernel/fire/rules.rs`). `collect_derived` reused.
- `tests/rete/probe_arc278_7strat_native_differential.rs` — the RED differential (already committed).

## The acceptance chain

```
clj+clara (neg.clj)  ──▶  wat+rete (fire-rules-spec)  ──▶  wat+rust-rete (fire-rules)
     Bad=1, Ok=1              Bad=1, Ok=1                     Bad=1, Ok=1   ← GREEN when all three agree
```
Plus: full `cargo test --release -p wat --test rete` stays green (172 → 173 with this probe), and the existing
P4a/deep-cascade/P4c/P6 differentials (native==oracle on non-stratified shapes) are unregressed.
