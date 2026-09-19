# SCORE — A4: the fixpoint's dedup set has one door

Behaviour-neutral. One owner. Two `&mut` handles became one. The cross-path hazard is **NOT live** and this type does not cure it. Floor GREEN. No `src/value/`.

## Scorecard

| # | result |
|---|---|
| 1 ★ behaviour-neutral | **HOLD.** `prod:derivations = 40000` (`fanout_rhs_key_alloc_census`) and `seen_facts = 41200` (`accum_alpha_memory_shape`) quoted before the change and asserted on the final floor. Dedup did not move. |
| 2 ★ one door | **HOLD on the dedup set.** `grep -rn 'seen_ids\|seen_rest' src/` → `node.rs:173,177` only, an unrelated `HashSet<i64>` of child node ids. The fixpoint halves are `SeenSet { ids, rest }`, private, not reachable as `seen_ids`/`seen_rest`. |
| 3 ★ the bypass does not compile | **HOLD.** Probe in `src/rete/kernel/tests/seen_set_bypass.rs` (outside `fire/delta.rs`). Quoted below. File and `mod` line deleted; `tests/mod.rs` diff-empty vs HEAD. |
| 4 ★ the guard, or its refusal | **HOLD as STOP-2.** No `debug_assert` shipped. Reason is in `SeenSet::insert`'s rest arm: checking an unstamped Aggregate is absent from `ids` needs the stamp `from_parts` would have written (hash of nature, class, fields) on every rest-arm insert. That is a hot-path hash even in debug. |
| 5 ★ severity honest | **HOLD.** Hazard is NOT live: `AggregateValue::from_parts` (`value.rs`) is the sole stamping site; `if id == 0 { 1 }` sentinel. Encapsulation does **not** cure an unstamped-but-shallow aggregate still taking `rest` inside the door. Hygiene plus a refused guard. |
| 6 hand-summed cardinality gone | **HOLD.** `round_census.rs:132` reads `seen.len()`. `len` is `#[cfg(test)]` because its only callers are the census instrument and cost tests; release would `dead_code` it. |
| 7 floor | **HOLD.** Final `.floor/2026-09-07T03-33-27Z/`: `Summary [ 458.478s] 5471 tests run: 5471 passed (2 slow), 21 skipped`. Count unchanged from HEAD (5471); no new tests. First floor captured red, not re-run — see below. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 1 is the whole claim** — a refactor that changes a value is a failed refactor. Row 5 is what stops the type reading as a correctness fix it is not.

## What landed

`SeenSet` in `src/rete/kernel/fire/delta.rs` owns both halves. Doors: `with_capacity`, `insert` (today's `seen_insert` body, unchanged), `len` (the old hand-sum), `reserve` (stamp half only — production's upper bound is stamped facts). `seen_insert` is gone. `RoundScratch.seen: &'a mut SeenSet`. Call sites in `alpha.rs`, `production.rs`, `round_census.rs`, eight `RoundScratch` literals in `delta.rs`, and the timing arms in `accum_alpha_cost.rs` / `accum_cost.rs` / `gather_probe_cost.rs`.

`reserve` still sizes only `ids`. That is the production call, named.

## Row 3 — bypass from outside the owning module

`kernel/tests/seen_set_bypass.rs` reading `s.ids` and `s.rest` on a `&super::SeenSet`:

```
error[E0616]: field `ids` of struct `delta::SeenSet` is private
 --> src/rete/kernel/tests/seen_set_bypass.rs:3:16
  |
3 |     let _ = &s.ids;
  |                ^^^ private field

error[E0616]: field `rest` of struct `delta::SeenSet` is private
 --> src/rete/kernel/tests/seen_set_bypass.rs:4:16
  |
4 |     let _ = &s.rest;
  |                ^^^^ private field
```

Without the `mod seen_set_bypass;` line in `tests/mod.rs` the file is not compiled and the probe is green vacuously — that was the SlotZip mistake. Driven with the `mod` line, then both deleted.

## STOP-2 — why there is no debug_assert

The invariant is "no logical value in both halves." The only suspicious case is an `Aggregate` taking the `rest` arm. Proving it is absent from `ids` requires the construction fingerprint `from_parts` writes. Computing that fingerprint on every rest-arm insert is a hash of `(nature, class, fields)` on the hot path, including debug. DESIGN said report the cost rather than ship that. The reason sits in the rest arm. The split itself is unchanged.

The hazard remains inside the door for an unstamped-but-shallow aggregate. `from_parts` being the sole stamp is a `src/value/` invariant rete does not own. This strike makes rete stop depending on two `&mut` handles agreeing; it does not enforce the stamp.

Arc 109's note (`4b8c987c8`) was not touched.

## First floor (captured, not re-run)

`.floor/2026-09-07T03-20-25Z/`: `Summary [ 459.834s] 5471 tests run: 5469 passed (2 slow), 2 failed, 21 skipped`. Arms:

1. `rete_engine_label_names_its_evidence::every_engine_label_names_its_evidence` at `tests/lint/rete_engine_label_names_its_evidence.rs:800` — `gather_probe_cost.rs:285` named `delta::seen_insert`, which is no longer a production definition. Retargeted to `engine: delta::SeenSet::insert`. The P/V arms call it as UFCS (`SeenSet::insert(&mut seen, f)`) because the gate's call scanner does not record a method call through a dot.

2. `binding_repr_bench::token_bindings_representation_dominance` at `src/rete/kernel/tests/binding_repr_bench.rs:762` — assertion `(3)+(4) THE LARGE END`: at cardinality 64 the array EXTENDED faster than the trie (3995.9 ns vs 5860.1 ns). This bench does not touch `SeenSet`. Named, not dismissed, not re-run. It passed on the final floor.

## Final floor

`.floor/2026-09-07T03-33-27Z/`: `Summary [ 458.478s] 5471 tests run: 5471 passed (2 slow), 21 skipped`.

## What this did not do

Did not merge the halves. Did not change what is deduped. Did not touch `src/value/`. Did not ship a hot-path hash under `debug_assert`. Did not treat encapsulation as a cure for the cross-path hazard.
