## STRUERE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

Read all 25 target files in full (23,886 lines). `src/rete/reachability.rs` was swept only for scope-boundary confirmation (out of scope per ruling 3).

I personally read `mod.rs` and all five `wat/` files directly; the twenty remaining Rust files were split across four parallel forks (inheriting this ward's full text and the scoping rules), whose findings I independently re-verified against the source myself before including them below — no forked claim is reported without my own `file:line` read.

**panic!/.unwrap()/.expect() surface**: grepped every target file, then classified each hit as test-only vs production by locating the `#[cfg(test)]` boundary per file. Result: **zero** production-code `panic!`/`.unwrap()`/`.expect()` reachable from user rule text anywhere in target 2 — a sharp contrast with target 1's 11 join-key `panic!` sites. The only non-test hits (`export.rs:1985,1997` `.expect("sorted key")`) are provably safe same-map lookups (key collected from the map, then immediately re-read, single-threaded).

**Boolean-classifier surface**: grepped every `fn ... -> bool` across all 20 Rust files (~30 functions) plus wat `-> :wat::core::bool` fns, and traced each to its actual callers, hunting for the target-1 `ClassPlan::observe`-shaped conflation (two facts collapsed into one boolean). All were verified honest against their callers — none found.

**`rune:struere`**: `grep -rn 'rune:struere'` across all 25 target files and, for context, the whole repo — **zero** occurrences anywhere in target 2 (repo-wide occurrences are all in `src/rete/kernel/**`, `src/macros/`, `src/types.rs`, `crates/wat-reader/` — out of scope).

## FINDINGS

**1. `src/rete/where_tree.rs:268-275` vs `:297-303` — doc asserts a stronger safety contract than the code implements**

`walk`'s doc says: *"The moment the walk takes a wildcard or a range edge — a guard, not a proof — everything below it is `maybe`"* (lines 271-272). The wildcard arm (line 316) does force `proven=false`, matching the doc. The range-edge arm does not: when `range_holds` returns `Some(true)` (line 299), `walk` recurses with the **incoming** `proven` unchanged — a satisfied range edge does *not* downgrade to `maybe`, contradicting the doc's blanket claim.

I traced why the code is actually safe despite the doc being wrong: the sole caller (`src/rete/kernel/fire/mod.rs:2284`) only treats an id as skippable when `proven.contains(&tid) && sink.where_tree.is_pure_cmp(tid)` — a **second**, undocumented-on-`walk` condition. For a `pure_cmp` id, `DimCon` allows exactly one constraint per dim (`where_tree.rs:74-80`), so proving every dim on the path (including a held range edge) does prove the whole AND. But `walk`'s own doc describes `proven` as if it were self-sufficient, and the tree indexes *every* compiled `where`, not just `pure_cmp` ones — a maintainer reading only this doc could "fix" line 299 to match the stated contract (forcing `proven=false` on any range edge), which would silently defeat the pure-cmp fast path for every range-typed clause, or could reasonably distrust the code without knowing the caller's second gate is what makes it sound.

- Lens: type/doc-doesn't-enforce.
- Level: **1** (the doc actively describes behavior the code does not have).
- Direction: state the real invariant on `walk` itself — a held range edge preserves `proven` because `proven` is only load-bearing in conjunction with `is_pure_cmp` at the call site — rather than leaving that half of the contract to be found in a different file.

**2. `src/rete/compiled_cond.rs:1547-1614` — an "exhaustive by construction" test fixture silently omits a real `Op` variant**

The test module's `lands()` (1550-1565) is a real, compiler-enforced exhaustive match over `Op`'s 8 variants (no catch-all), classifying `Op::Bind | Op::Eval → Driver`, everything else `→ Core`. `every_op_variant_lands_in_core_or_driver` (1567-1614) claims this coverage is locked (*"Freeze next to `Op`: a new variant that does not compile is a red build"* — line 1539) but drives it via a hand-written `variants` array (1571-1593) that lists only 7 literal `Op` values — **`Op::Eval { .. }` is missing entirely**. The assertion `driver.len() == 1, "driver must be exactly Bind"` (1608-1612) only holds *because* `Eval` was left out; per `lands()`'s own logic, adding it would make `driver.len() == 2` and break the assertion as written.

Compounding it: the file's own top-of-module doc (lines 95-96, predating `Eval`'s introduction under "FIX-LIST F") still says *"Driver = slot population (`Bind` only)"* — disagreeing with `lands()` itself (line 1557: `Bind | Eval → Driver`), which the test fixture's `driver.len() == 1` assertion also happens to (wrongly) agree with. Three sources — an old module doc, a hand-maintained test array, and the live classifier — now describe two different truths, and the test that's supposed to be the freeze is asserting the stale one.

- Lens: composition-doesn't-hold / type-doesn't-enforce (the match's real, compiler-checked exhaustiveness does not transfer to the array literal, which is not compiler-checked, and nothing here proves the array covers `Op`).
- Level: **2** (mumble — the invariant claimed is not the invariant actually tested).
- Direction: derive `variants` from something exhaustive over `Op` (a macro, or a match with `_ => unreachable!()`) instead of a hand-typed array, and fix the stale "Bind only" doc at lines 95-96 to match `lands()`.

**`rune:struere`**: none present anywhere in target 2 to render a verdict on.

## FINDINGS — 2
