# EXPECTATIONS — an unverified link is a citation nobody can check

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,230 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `7ead5953e`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | doc attribution | **3 blocks on `SessionMemoryCeilingExceeded`**; `RuleSetMayNotTerminate` and `FixpointRoundCapExceeded` have **none** (driven) | one block per variant, four documented |
| 2 | the matchability justification | attached to a different failure | on the failure it justifies |
| 3 | `signal.rs` links | **9 unresolved** (driven) | **0** |
| 4 | ★ the gate exists | nothing runs rustdoc; the lint is not enabled (both greps empty) | a gate that runs it and compares a **named list** |
| 5 | the list is named, not counted | — | entries identify each link; **no bare count** |
| 6 | the gate can FAIL both ways | — | driven: add a broken link → RED; fix a listed one without delisting → RED |
| 7 | tree-wide total | **50 unresolved** (measured) | **41** ± your own count — report it |
| 8 | radius | — | `signal.rs` + one `tests/lint/` file |
| 9 | lints | **116/116** (measured) | green |
| 10 | floor | **5230/5230** (measured) | ≥ 5,230, zero FAIL rows |
| 11 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — the gate must fail in BOTH directions

1. **Add a broken link** in a file not in the list → the gate must RED and name it. *A gate that only
   catches removals is a gate that lets the class grow.*
2. **Fix a listed link without removing its entry** → the gate must RED. This is the direction the
   `purity.rs` ledger records as the one a cardinality check misses, and it is why row 5 exists.
3. **Restore both** → green.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

50–70 minutes. The doc split is mechanical; the gate and its two-direction mutation are the work, and
trap 3's timing question may change its shape.

## What would make this strike a failure even if every test passes

**A count-based ratchet.** `purity.rs`'s own doc records what that costs: a new broken link walks in
free whenever someone fixes an old one, and the gate cannot name the offender. Rows 5 and 6.2.

The second: **splitting the docs but not adding the gate.** Four variants would read correctly and
the next move of a variant breaks them again in silence — which is exactly what E4 did to this file
one commit ago, with a full floor and a clean clippy.
