# SCORE — the probe's ★ paragraph asserted a fence that does not exist

Struck, not deleted. A rete defn whose body constructs a record is admitted. The probe still
extracts; construction is already driven elsewhere. Comment only.

## Scorecard

| # | result |
|---|---|
| 1 ★ the evidence re-runs | **HOLD.** Scratch fixture printed `COMPILE: Compiled` and completed. Quoted below. |
| 2 ★ comment-only | **HOLD.** `git diff -U0` — every `+`/`-` content line begins `;;`. |
| 3 ★ the probe still passes | **HOLD.** All six `probe_arc278_then_user_forms` tests green, including both `userfn_head_item_*`. EXPECTATIONS filter `test(then_user_forms_userfn)` matches nothing; those are the names. |
| 4 ★ the false sentence is gone | **HOLD.** `grep -c 'is refused today'` → 0. |
| 5 ★ both drivings cited | **HOLD.** `2026-08-28` and `2026-09-07` both present (3 hits). |
| 6 ★ floor | **HOLD.** `.floor/2026-09-07T23-57-15Z/`: `Summary [ 487.523s] 5480 tests run: 5480 passed (4 slow), 19 skipped`. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is why the correction is allowed to land.**

## STOP-1 did not fire

`cargo run --release --bin wat -- wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat`:

```
"COMPILE: Compiled"
"ORACLE STRATA:"
{"a2::Out" 1 "a2::Rate" 1}
"ORACLE rule-produces via:"
#wat.core/PersistentVector ["a2::Rate"]
"NATIVE facts [Bad Rate Out]:"
#wat.core/PersistentVector [0 1 1]
"ORACLE facts [Bad Rate Out]:"
#wat.core/PersistentVector [0 1 1]
```

`:a2::mk-rate` is a `:wat::rete::core::defn` whose body constructs `(:a2::Rate :count k)`. Compile
succeeds. The `COMPILE: Compiled` line in `main` is a print, not a compile-outcome match; the
program then calls `:a2::staged` which assertion-fails on `MayNotTerminate`. Completing is the
proof the construction path compiled.

## STOP-2 did not fire

`:tf::first-rate` extracts: `PersistentVector/first` on the accumulate bind. The `:undefined`
arm holds a literal `(:tf::Rate :count 0)` as the empty-vector default — a fallback, not the
function's work. The probe's subject remains a user-fn `:then` head.

## What the paragraph now carries

1. The claim is struck (⛔ STRUCK 2026-09-07), not silently deleted.
2. Why it was believed: three probes, each a plain `:wat::core::defn`, so Law A refused the FN
   three times, read as three fences.
3. Both drivings: `stratify.rs:418-424` (2026-08-28, named as the kwargs-construct row above
   `rete_fn_body_mints`) and the scratch fixture (2026-09-07, behind `21a5f8514`).
4. What actually guards a minting body: `rete_fn_body_mints`, pinned by
   `probe_arc278_termination_fn_head.wat`.

No new claim about `purity.rs`. Extraction stays; rewriting `:tf::first-rate` to construct was
REJECTED.

## What this did not do

No `:then` rewrite. No `purity.rs`. No gate widening (`tests/rete/*.wat` prose is still in
neither `no_stale_path_in_doc` nor `rete_citation_resolves` — rowed in the DESIGN, not this
strike).

## Final floor

`.floor/2026-09-07T23-57-15Z/`: `Summary [ 487.523s] 5480 tests run: 5480 passed (4 slow), 19 skipped`.
