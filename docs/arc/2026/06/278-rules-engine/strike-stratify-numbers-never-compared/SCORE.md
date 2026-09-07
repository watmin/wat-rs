# SCORE — the two stratifiers disagree on numbers; facts still agree

Measurement. Native `Tally` is stratum **1**; oracle MEASURE is `{}` (stratum **0**). Keys agree.
Both halves are now driven **in the same test, same process, same binary**.
`probe_arc278_derived_exists_acc` is still GREEN on facts. False lockstep claim in two headers,
not a correctness defect. No `+1` added or removed. Floor GREEN.

The first SCORE's oracle half was a printed constant. REVIEW refuted that; this file is the
whole gate.

## Scorecard

| what | result |
|---|---|
| oracle, in-process | **HOLD.** `:wat::rete::stratify` via `eval_in_frozen` in the same frozen world. ANCHOR `l23::Ok2=1`; MEASURE `{}`. Matches the scratch `.wat`. REVIEW STOP-1/STOP-2 did not fire. |
| native MEASURE | **HOLD. Tally ⇒ 1.** Reading at `stratify.rs:221-227` confirmed, not refuted. |
| AGREEMENT arm | **HOLD.** Both engines raise `l23::Ok2` to 1. |
| DIVERGENCE arm | **HOLD.** `native_bag["l23::Tally"] == Some(1)` and `oracle_bag["l23::Tally"] == None`. |
| key spelling | **HOLD.** Both sides spell `l23::Ok2` / `l23::Tally` with no leading colon. |
| `derived_exists_acc` | **HOLD.** 3/3 pass, untouched. |
| floor | **HOLD.** `.floor/2026-09-07T07-52-33Z/`: `Summary [ 488.009s] 5473 tests run: 5473 passed (3 slow), 22 skipped`. Count unchanged (same test, oracle half now driven). |
| clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

The prediction was `Tally ⇒ 1`. It held. The finding is DESIGN reading 1: **the strata differ but the facts do not.**

## The numbers, driven in one process

```
NATIVE ANCHOR keys (raw): ["l23::Ok2" 1]
ORACLE ANCHOR keys (raw): ["l23::Ok2" 1]
NATIVE MEASURE keys (raw): ["l23::Tally" 1]
ORACLE MEASURE keys (raw): []
bag exists_and_from_types: [[], ["l23::Ok"]]
```

Hardcoded `ORACLE …` format-string literals are gone. The scratch `.wat` is the same source
`include_str!`'d into the test; the oracle verb is `:wat::rete::stratify` on that world.

`exists_and_from_types` on the tally rule is `["l23::Ok"]`. `ok` produces `Ok` and does not bag it, so `derived` is true and the native `+1` fires. The oracle folds that `:from` into `rule-consumes` and `req-pos` is NOT +1, so `Tally` stays 0 and is not recorded.

## What a green on facts cannot see

`probe_arc278_derived_exists_acc` asserts `oracle must match native on derived exists/acc` and is GREEN (3/3). That comparison is query row counts. Both engines derive the same `Tally`; they just number its stratum differently. Stratify is the ordering layer, not the supersession layer — the oracle's own header said so. The defective thing is the lockstep claim in `stratify.rs:205` ("Mirrors `stratify-sweep`") and the oracle's "lockstep with native `rule_consumes`" sentence covering `:exists` / acc `:from`.

No cure in this strike. No `+1` added or removed.

## First floor (captured, not re-run)

`.floor/2026-09-07T07-20-39Z/`: `5473 tests run: 5472 passed, 1 failed`. Arm: `rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves` at `tests/lint/rete_citation_resolves.rs:566` — `stratify_numbers.rs:6` cited `derived_exists_acc`, which is not an identifier. Spelled `probe_arc278_derived_exists_acc`.

## Final floor

`.floor/2026-09-07T07-52-33Z/`: `Summary [ 488.009s] 5473 tests run: 5473 passed (3 slow), 22 skipped`.

## What this did not do

Did not touch `stratify.rs` or `stratify.wat`. Did not touch `probe_arc278_derived_exists_acc`. Did not write a parallel view-builder. Did not normalise keys. Did not treat a facts-green as lockstep on numbers. Did not restore hardcoded oracle values.
