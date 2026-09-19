# Intueri Cast Report — `src/rete/kernel/**` (excl. tests) + `wat/rete/oracle/**`

## Scope actually read

All 22 Rust files under `src/rete/kernel/` (excluding `tests/`) and all 6 `.wat` files under `wat/rete/oracle/`. Read in full: `mod.rs`, `outcome.rs`, `node.rs`, `insert.rs`, `census.rs`, `arm.rs`, `stratify.rs`, `fire/pass/mod.rs`, `fire/pass/root_join.rs`, `fire/pass/production.rs`, `fire/pass/round_census.rs`, `fire/pass/join_after_filter.rs`, `fire/pass/accumulate.rs`, `fire/pass/alpha.rs`, `fire/pass/filter.rs`, `fire/pass/filter_after_join.rs`, `fire/pass/hash_join.rs`, `wat/rete/oracle/insert.wat`, `wat/rete/oracle/explain.wat`. Read substantially (headers, all struct/type definitions, and multiple full functions, generally 60-100% of the file): `session.rs` (~1850/1848 via two large excerpts covering the core types + the full function index), `fire/mod.rs` (module header + FireCtx/JoinAlpha + alpha_pass/root_join_pass + the whole GatherIntern/col_field family), `fire/delta.rs` (full `fire_fixpoint_delta_armed`), `fire/rules.rs` (header + `fire_rules_stratified` opening), `wat/rete/oracle/fire.wat`, `wat/rete/oracle/pass.wat` (~450/810 lines), `wat/rete/oracle/accum-pass.wat` (~90/323 lines), `wat/rete/oracle/stratify.wat` (header). `fire/acc.rs` and the remainder of `stratify.wat` were surveyed via full function-signature listings but not read line-by-line.

## Overall impression

This is, by a wide margin, one of the most communicatively disciplined codebases I have cast this ward over. Nearly every non-trivial function, type, and thread-local carries a WHY comment, and the codebase has a visible habit of *correcting its own prior false claims in place* rather than silently fixing them — e.g. `fire/mod.rs`'s header ("do not learn a rule from the `*_pass` suffix... an earlier version of this header did, and it was wrong in both directions"), `arm.rs`'s `DerivedIndices` doc (explicitly records that `intueri` found a coverage gap on 2026-08-30 and states exactly what is and isn't true today), `census.rs`'s narrowing note ("AND THE REASON GIVEN FOR NARROWING IT WAS WRONG"), and `stratify.rs`'s repeated "⛔ WHAT THIS PARAGRAPH USED TO SAY WAS WRONG" corrections. Names are domain nouns throughout (`RuleDep`, `CondDriver`, `AccFold`, `InternedNetwork`, `ArmLease`, `JoinLeftIndex`/`JoinRightIndex`, `BetaStore`, `RoundScratch`, `FireCtx`) and abbreviations (`el`, `vid`, `bm`, `tok`, `off`) are consistently short-scoped and typed. WHY comments overwhelmingly outnumber WHAT comments. I found no lying names, no `utils`/`helpers`/`manager` modules, and no stale TODOs (grep for TODO/FIXME/XXX over the target returned nothing).

## Findings

**1. `src/rete/kernel/fire/pass/accumulate.rs:18-19` — Level 2 (mumble), doc comment incomplete**

```rust
/// Dispatch the accumulate nodes over this round's delta.
pub(crate) fn accumulate_pass(
```

The function's body (lines 36-89) also performs "3.20 PRE-DISPATCH": pulling forward any `Test` node that is a parent of an `Accumulate` node, dispatching it early via `dispatch_where_tests`, and recording it in `pre_dispatched` so pass 3.5 (`filter_pass`) knows to skip it. This is a real, load-bearing second responsibility (the surrounding inline comments explain it was the fix for a real bug where a rule silently matched zero), but the function's own one-line doc promises only "dispatch the accumulate nodes." A reader who trusts the doc comment and skims past the pre-dispatch block would not learn that `accumulate_pass` is also where a sibling pass's inputs get seeded.

Suggested direction: extend the doc comment to name both responsibilities (e.g. "Pre-dispatch Test parents that feed an Accumulate (3.20), then dispatch the accumulate nodes over this round's delta (3.25)."), or split the pre-dispatch block into its own named function called from the fixpoint loop alongside this one — the file already treats 3.20 and 3.25 as distinct pass numbers in its own comments.

## Soft/contextual observation (not a violation, reported for completeness)

Several `fire/pass/*.rs` functions — `accumulate_pass`, `filter_pass` (`filter.rs`), `filter_after_join` (`filter_after_join.rs`), and `hash_join_delta` (`hash_join.rs`) — run 250-370 lines with local nesting past 3 levels in places. None of these carry a formal `rune:intueri(length)` or `rune:intueri(complexity)`. However, every one of them is preceded by a module doc explicitly justifying the shape: these are verbatim, mechanically-adapted extractions from a single 1774-line fire loop (`DESIGN-STONE-partire-fire-loop`), deliberately kept as literal moves ("no clone removed, no name improved, no comment rewritten in the same commit as a move") so the extraction diff stays reviewable, with the genuinely worst nesting (`hash_join_delta`'s "nesting NINE, the deepest point in the whole rete engine") already split out into `hj_step3_term1`/`hj_step4_term2`. This reads as the "orchestration sequence has a reason" exemption the ward itself describes, just expressed in prose at the module level rather than a per-function rune. I'm not filing it as a finding, but flagging it in case the ward wants the exemption formalized with `rune:intueri(length)` markers on the four functions named above.

## Runes encountered

Only one rune in this target uses this ward's own `rune:intueri(...)` syntax:

- **`wat/rete/oracle/fire.wat:54`** — `rune:intueri(naming) — oracle populate-then-emit walker (acc+filter+hash-join); the name is the historical walk-sorted-ids split, not filter-alone. Rename would fork every oracle fire caller.` — **Verdict: clear.** `walk-filter-ids` (defined lines 57-76) does in fact dispatch `accumulate-pass`, `filter-pass`, and `hash-join-pass` in sequence, not filtering alone; the rune accurately names the mismatch and gives a concrete, checkable cost for renaming (every oracle fire caller). This is exactly the kind of weighed, justified exemption the ward's rune mechanism exists for.

All other runes I encountered in this target belong to other wards' vocabularies (`rune:temperare`, `rune:struere`, `rune:sequi`, `rune:perspicere`, `rune:circumspicere`, `rune:lint`) and are out of scope for intueri's own verdict; I did not evaluate their reasons. For the record, sites seen: `fire/pass/root_join.rs:53` (`temperare`), `fire/pass/filter.rs:57` (`temperare`), `census.rs` (multiple `sequi`/`perspicere`), `node.rs:83` and `session.rs:20,61,85,97,109` (`struere`), `arm.rs:705,717` (`sequi`, `circumspicere`), `fire/pass/production.rs:7`, `fire/pass/alpha.rs:93,169`, `fire/pass/hash_join.rs:21` (`lint(cited-name-absent)`), `fire/mod.rs`'s `GatherIndex::of` region and `pass.wat:245` (`perspicere`).

## Summary

One Level-2 finding (`fire/pass/accumulate.rs:18-19`), one clear rune, no lies, no dead-referent names, no stale comments, no `utils`/`manager` mumble modules. The target reads as a codebase where the spark is very much alive, including a visible institutional habit of catching and correcting its own prior communication failures in place.
