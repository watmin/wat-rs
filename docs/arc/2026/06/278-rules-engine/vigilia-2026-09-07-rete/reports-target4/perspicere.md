# PERSPICERE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### 1. What I swept, and the delta from the handed-down 72

The handed-down number (**72**, from `grep -rhoE '<[^<>]*<[^<>]*<' tests/rete src/rete/kernel/tests | wc -l`) is reproducible verbatim. Breaking it down by extension:

```
rs:  36 occurrences (33 lines)
wat: 36 occurrences
edn:  0
bad:  0
```

**Both halves are noise, for different reasons:**

- **The 36 `.wat` occurrences are 100% false positives**, exactly as the brief warned: every one is `<-` (binding arrow) or `:wat::rete::core::i64::<` (comparison), e.g. `tests/rete/probe_arc278_P2_native_fire_once.wat:12`. Confirmed by reading all matches — no `.wat`/`.edn`/`.wat.bad` file uses `<` for type nesting anywhere in these trees. I did nothing further with them; the trigger does not apply.
- **The 36 `.rs` occurrences are ALSO 100% false positives**, but for a different reason: the pattern `<[^<>]*<[^<>]*<` requires **3** unbroken `<`, which is stricter than the ward's own stated threshold ("2 or more `<`"). What it actually caught were: `.rs` string literals holding embedded `.wat` source text (e.g. `probe_arc278_8custom_native_differential.rs:34`), doc-comments quoting wat syntax (`probe_arc278_P12_explain_walk.rs:24`), and `println!` alignment specifiers like `{:<2}{:<6}` (`node_share_cost.rs:1049-1050`). **Zero real Rust generic-type nesting matched the given pattern at all.**

So the "72" — while an honest count of its own regex — corresponds to **zero genuine candidates**. I re-derived using the ward's actual threshold: two adjacent `IDENT<...IDENT<` (real Rust generic syntax, excluding strings/comments), which found **55 lines across 17 `.rs` files**. I read every one of those 55 lines in context (not sampled — small enough to fully cover) plus their surrounding 5-10 lines to judge subject-vs-scaffolding and to check for runes.

What I did **not** read line-by-line: the ~87 `.rs` files that never matched any candidate pattern (population too large to hand-audit in full; the two independent greps — 3-open and 2-open identifier patterns — are the sampling instrument, and I trust their negative on files neither flagged).

### 2. The 12 runes — recorded as skipped, not re-adjudicated

Confirmed all 12 present, exactly as given, via `grep -rn "rune:perspicere" tests/rete src/rete/kernel/tests`: `gather_probe_cost.rs:498,508,526,649,660,784`; `probe_arc278_export.rs:511`; `alpha_discrimination.rs:74,252,402`; `mod.rs:500`; `accum_alpha_cost.rs:819`. No `.wat` sibling in either target tree carries a `perspicere` rune (checked — these test files have no `.wat` siblings at all, so the placement-fact warning doesn't apply here). These 12 correctly cover the type expressions immediately below them; I did not reopen their verdicts.

### 3. Findings

**Finding A — 9 raw re-spellings of an existing production typealias (strongest finding).**
`HashMap<i64, Vec<u8>>` (2 `<`) appears unaliased at:
`src/rete/kernel/tests/accum_alpha_cost.rs:242,272,586,1161,1191,1295,1308,1350` and `src/rete/kernel/tests/gather_probe_cost.rs:999`.
A sibling alias **already exists** and is used pervasively in production: `src/rete/kernel/session.rs:176: pub(crate) type BindOnlyFields = HashMap<i64, Vec<u8>>;` (consumed at `session.rs:596`, `fire/pass/hash_join.rs`, `fire/pass/filter.rs`). The test-side variables are even named `bind_only`/`bind_only_prod`/`bo` — the same name as the production field — yet none of the 9 sites import `BindOnlyFields` (`grep -rn "BindOnlyFields" tests/rete src/rete/kernel/tests` → empty). Role-noun the type is asking for: exactly `BindOnlyFields`, which already exists. **Recommendation: mint via reuse — import and use `super::BindOnlyFields` instead of respelling.** Subject vs scaffolding: these locals directly instantiate the production domain type and are what the accumulate-cost benchmarks measure over — closer to test subject than throwaway scaffolding, which strengthens the case for reuse over a rune.

**Finding B — unruned siblings of an already-validated read-once shape, same file.**
`src/rete/kernel/tests/gather_probe_cost.rs:552` (`FxHashMap<super::JoinKey, Vec<usize>>`), `:553` and `:688` (`FxHashMap<Value, Vec<usize>>`), `:695` (`FxHashMap<u32, Vec<usize>>`) — the identical "gather microbench index" shape that carries `rune:perspicere(read-once) — gather microbench index; not a domain noun` at 5 sibling sites in the *same file* (`499,509,527,650,661`, verified by reading the runed lines directly above each). These 4 have no rune above or on the line. This is an **inconsistency, not a fresh design question** — the excusare-validated verdict for this exact shape in this exact file was already "rune, not alias" (notably, even where production aliases `GatherUnary`/`GatherNary` at `fire/mod.rs:1608-1609` would technically fit, the file's own precedent rejected reuse). **Recommendation: add the matching rune to the 4 missing sites** for consistency with their 5 siblings.

The same "read-once local index" shape recurs a third time, unruned, at `src/rete/kernel/tests/accum_cost.rs:495-496, 513-514, 531-532` — three near-identical closures each declaring `HashMap<&str, Vec<&Value>>` (return type + local `let idx`), built and consumed once within one test. Same disposition: rune(read-once), not alias.

**Finding C — deepest genuine nesting found (4 `<`), unruned.**
`tests/rete/probe_arc278_rete_defn_recurse.rs:165`: `let mut runs: Vec<(Option<i32>, Vec<u8>, Vec<u8>)> = Vec::with_capacity(MUTUAL_RUNS);` — role-noun: something like "per-run outcome (exit code, stdout, stderr)". Built once, consumed once, within a single `#[test]` fn proving cross-run blame stability (subject of the test is process-blame determinism, not this collection's shape — scaffolding). No sibling alias found. **Recommendation: rune(read-once)** — a name here would be single-use ("RunOutcome"/"SubprocessRuns") and per the ward's own guidance that's exactly what the rune, not a mint, is for.

**Weaker candidate, human judgment:** `tests/rete/probe_arc278_compiled_where_ops.rs:102`: `let mut per_file: HashMap<String, (usize, HashMap<String, usize>, usize)> = HashMap::new();` — function-local census accumulator (filename → (count, per-key counts, count)), single function, already explained by surrounding prose and the variable name. Matches the "one-shot census bag" shape validated elsewhere. Leaning rune(read-once), but marginal enough I'd leave it to a human rather than assert it.

**Considered and explicitly NOT flagged** (mechanically cross the 2-`<` count but fail the substantive "hidden noun" test — the ward's reading principle, not just its counting rule):
- `Result<Vec<i64>, String>` at `probe_arc278_concurrent_retes.rs:48`, `probe_arc278_leading_filter_multiplicity.rs:39`, `probe_arc278_7strat_native_differential.rs:22` — idiomatic fallible-vector return, reads in one breath, no missing noun.
- `Vec<Vec<Value>>` at `src/rete/kernel/tests/strat_cost.rs:167,279,556` — local stratified-fixture builders (`derived`/`strata`), self-documenting variable names, single-function scaffolding.
- Tuple-of-siblings return types (`(Vec<UniformAxis>, Vec<String>)` at `where_tree_branch_differential.rs:428`, `(Vec<String>, Vec<String>)` at `:126`, `(Vec<super::Element>, Vec<(u32,u32)>)` at `gather_probe_cost.rs:336`) — two independent 1-level `Vec`s counted together by the character-count rule, but not actually a buried noun.
- `right_index_counter_invariant.rs:215,219` — `PrefixRow`/`PrefixMiss` are typealias *bodies* — explicitly exempt.
- `accum_cost.rs:455` — `&Arc<Vec<String>>` closure parameter, single-use scaffolding, same shape already covered by the excusare-validated read-once precedent elsewhere.

### 4. What I looked for and did not find

- No case where a deep type is the literal *subject under test* of any `#[test]` (i.e., a test asserting on the type machinery itself) — the "test files exercising the typealias machinery" exemption clause does not apply anywhere in this population; every candidate is ordinary domain/scaffolding typing.
- No `.wat`/`.edn`/`.wat.bad` file contains real generic-type nesting of any kind — the language doesn't spell types this way, and no rune was needed or found there.
- No case where a rune's reason text was vague, copy-paste, or unjustified — all 12 read as genuine (not re-litigated, per instruction, but read).

**FINDINGS**
