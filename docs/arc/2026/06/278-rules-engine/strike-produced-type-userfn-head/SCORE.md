# SCORE — the oracle drops `Out` when `:then` is a user fn

Arm 1 names diverge as predicted. Arm 2 is constructible, and it changes FACTS.
Native `[Bad Rate Out] = [0 1 1]`. Oracle `[0 1 0]`. Clara 0.24.0 `[0 1 1]`.
The oracle is the wrong one. Neither stratifier was touched.

## Scorecard

| # | result |
|---|---|
| 1 ★ oracle re-confirmed | **HOLD.** `cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat`: ANCHOR `["pt::Rate"]`; MEASURE `["pt::first-rate"]`. |
| 2 ★ arm 1 native ANCHOR | **HOLD.** `pt::Rate`. Extractor is sound. |
| 3 ★ arm 1 native MEASURE | **HOLD.** `pt::Rate` against oracle `pt::first-rate`. STOP-1 did not fire. |
| 4 ★ arm 1 oracle driven | **HOLD.** `eval_in_frozen` of `:wat::rete::rule-produces` in the same frozen world. `include_str!` of the scratch. No hardcoded oracle print. |
| 5 ★ arm 2 facts differ | **HOLD. STOP-3.** Native Out=1, oracle Out=0. Rate exists on both. Oracle strata `{a2::mk-rate 1}` — `Rate` never raised, `Out` sat at 0 and was not re-fired. |
| 6 ★ Clara referee | **HOLD, now re-runnable.** First SCORE had no `.clj` on disk — REVIEW refuted that. Instrument: `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.clj`. Command and verbatim output below. `[0 1 1]`. Matches native. |
| 7 ★ floor | **HOLD.** `Summary [ 482.338s] 5475 tests run: 5475 passed (2 slow), 22 skipped`. `.floor/2026-09-07T10-43-43Z/`. +2 tests. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 no cure | **HOLD.** No `stratify.rs`. No `wat/rete`. No `purity.rs`. |

★ load-bearing. **Arm 2 is the finding. Arm 1 only names it.**

## Arm 1 — names, driven in one process

```
NATIVE ANCHOR: ["pt::Rate"]
ORACLE ANCHOR: ["pt::Rate"]
NATIVE MEASURE: ["pt::Rate"]
ORACLE MEASURE: ["pt::first-rate"]
```

Gate: `src/rete/kernel/tests/produced_type_userfn.rs`.

## Arm 2 — facts

Scratch `wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat`:

```
Src(1)
Bad :- Src, k=2                  (does not fire)
Rate :- Src, (not Bad), :then (mk-rate ?k)
Out  :- Rate
```

`mk-rate` is `:wat::rete::core::defn` whose body is `(:a2::Rate :count k)` — a bound symbol, not a List, so `rete_fn_body_mints` does not fire. Compile is `Compiled`. The purity-fence outcome 2 did not happen.

```
COMPILE: Compiled
ORACLE STRATA: {"a2::mk-rate" 1}
ORACLE rule-produces via: ["a2::mk-rate"]
NATIVE facts [Bad Rate Out]: [0 1 1]
ORACLE facts [Bad Rate Out]: [0 1 0]
```

Clara referee, re-run 2026-09-07 from the tree:

```
clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
        -M wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.clj
CLARA facts [Bad Rate Out]: [0 1 1]
```

Two modelling points, stated in that file's header: (1) the user fn is inlined as `(insert! (->Rate ?k))` because `mk-rate`'s body is exactly `(:a2::Rate :count k)`; (2) `Bad` never fires (`k=2` against `Src(1)`), so the negation is vacuously true and Clara's Rate is derived for the same reason wat's is.

The probe header that said a constructing user fn is refused with "`kwargs-construct` is not pure" is false for a rete defn whose constructor argument is a bound var. `stratify.rs:420-422` already recorded that (2026-08-28). This strike drove it. Rowed for its own strike; not fixed here.

## What this does not do

Does not change `rule-produces` or `produced_type`. Direction is now known — Clara agrees with native — and the cure is a later strike.

## Landing

Two tests, two scratch `.wat` files, one Clara `.clj` referee, one `mod` line. Do not commit unless asked.
