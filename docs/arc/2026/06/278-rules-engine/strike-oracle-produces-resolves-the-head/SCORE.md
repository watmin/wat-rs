# SCORE — the oracle's `rule-produces` resolves the head; `Out` is back

The colon-strip is gone. `rule-produces` uses `compile.wat`'s recipe. Oracle facts
`[0 1 1]`, matching native and Clara. ANCHOR unmoved. Floor 5475/0 fail — no other
test was asserting the wrong answer.

## Scorecard

| # | result |
|---|---|
| 1 ★ before | **HOLD.** `ORACLE facts [0 1 0]`. Strata `{a2::mk-rate 1}`. Quoted below. |
| 2 ★ after | **HOLD.** `ORACLE facts [0 1 1]`. Native `[0 1 1]`. |
| 3 ★ strata after | **HOLD.** `{"a2::Rate" 1 "a2::Out" 1}`. No `a2::mk-rate` key. `rule-produces via: ["a2::Rate"]`. |
| 4 ★ names after | **HOLD.** MEASURE `["pt::Rate"]`, was `["pt::first-rate"]`. |
| 5 ★ ANCHOR unmoved | **HOLD.** `["pt::Rate"]` on both sides. |
| 6 ★ gate flips | **HOLD.** Both tests assert AGREEMENT. Messages rewritten. ANCHOR left as the non-vacuity guard. |
| 7 ★ floor | **HOLD.** `Summary [ 485.374s] 5475 tests run: 5475 passed (3 slow), 22 skipped`. `.floor/2026-09-07T11-18-09Z/`. **No other test reddened.** |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 no fallback / no native touch | **HOLD.** Colon-strip deleted, not kept. `produced_type`, `rule-negates`, `rule-consumes`, `purity.rs` untouched. |

★ load-bearing. **Row 2 is the cure's acceptance. Row 7 is what says it did not re-pin the defect.**

## Before

```
"ORACLE STRATA:"
{"a2::mk-rate" 1}
"ORACLE facts [Bad Rate Out]:"
#wat.core/PersistentVector [0 1 0]
```

## After

```
"ORACLE STRATA:"
{"a2::Rate" 1 "a2::Out" 1}
"ORACLE rule-produces via:"
#wat.core/PersistentVector ["a2::Rate"]
"NATIVE facts [Bad Rate Out]:"
#wat.core/PersistentVector [0 1 1]
"ORACLE facts [Bad Rate Out]:"
#wat.core/PersistentVector [0 1 1]
```

Names scratch: ANCHOR `["pt::Rate"]` (unmoved); MEASURE `["pt::Rate"]`.

## The recipe

`eval-ast!` the head. If `type` is `"wat::core::fn"`, use it; else re-resolve through the
PRIME `:T'` keyword. Then `return-type-of` — already a colon-free FQDN. Same path for a
bare record head (keyword → PRIME → constructor → type). `eval-ast!` resolved at stratify
time; STOP-1 did not fire.

## Landing

`wat/rete/oracle/stratify.wat` (one fold body) and `produced_type_userfn.rs` (two
assertions + messages). Do not commit unless asked.
