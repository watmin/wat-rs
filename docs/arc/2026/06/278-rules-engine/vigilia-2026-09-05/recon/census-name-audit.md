# ward `census-name-audit` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## Census infrastructure

All named counters funnel through one module: **`/home/john/work/holon/wat-rs/src/rete/kernel/census.rs`**

| family | API | storage |
|---|---|---|
| op counts (`"name"` → u64) | `census_count` (census.rs:376) / `census_count_n` (census.rs:366) | `CENSUS_COUNTS` census.rs:360 |
| gather visits | `census_gather_visit` census.rs:237 | `GATHER_VISITS` census.rs:232 |
| phase timing + **pair count** | `phase_end` census.rs:332 (`e.1 += 1`) | `PHASE_NANOS` census.rs:314 |
| beta traffic | `beta_written` census.rs:425 / `beta_read` census.rs:439 | `BETA_TRAFFIC` census.rs:419 |
| right-index appends | `right_idx_appended` census.rs:532 | `RIGHT_IDX_APPENDS` census.rs:521 |
| per-round snapshot | `record_round_census` `src/rete/kernel/fire/pass/round_census.rs:26` | `FIRE_CENSUS` census.rs:98 |

31 distinct string keys, 39 increment sites. I read every one. Snapshot fields in `RoundCensus` (alpha_nodes/alpha_elements/beta_tokens/network_edges/seen_facts…) all check out — names match the sums at round_census.rs:90-114.

---

## A. `filter:test-pass` — name says "a test passed", quantity is passes ∪ elided-passes, and a consumer subtracts it from evals (HIGHEST)

**Name/doc:** `/home/john/work/holon/wat-rs/src/rete/kernel/tests/where_tree_branch_differential.rs:83` — *"`filter:test-pass` — a token that reached a beta/d_beta push."* No doc at any increment site.

**Increment sites** (`/home/john/work/holon/wat-rs/src/rete/kernel/fire/mod.rs`):
- `2067: census_count("filter:test-reuse");`
- `2068: census_count("filter:test-pass");`  ← **reuse arm: `exec_stashed_where` is never called, no eval happened**
- `2076: census_count("filter:test-evals");` → `2078: census_count("filter:test-pass");`
- `2090: census_count("filter:test-evals");` → `2102: census_count("filter:test-pass");`

**Verdict:** `test-pass` is a UNION of two disjoint populations (evaluated-and-passed, plus tree-proven-pushed-without-evaluating), so it is not a subset of `filter:test-evals` — yet `/home/john/work/holon/wat-rs/src/rete/kernel/tests/node_share_cost.rs:867` does exactly that subtraction:
`867: let wasted = evals.saturating_sub(passes);`
and gates on it at `node_share_cost.rs:882` (`evals <= passes.saturating_mul(2)`) and `node_share_cost.rs:894` (`worst_waste < 50.0`). On this axis `evals == 0` and `reuse > 0` (asserted at node_share_cost.rs:298), so `passes` is 100% reuse, `wasted` saturates to 0, and "0% waste" — the flattering reading — is structural, not measured. Same shape as `compiled:calls`: one key, two units.

## B. `compiled:calls` — residual non-call site survives the C14 rename

**Name/doc:** `/home/john/work/holon/wat-rs/src/rete/compiled_cond.rs:912-914` ("the compiled path's call counter"); the fix note at `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/alpha.rs:195-203` blesses "the two genuine per-call sites (`fire/delta.rs`'s `skip_span` arm and `compiled_cond.rs`)".

**Sites:**
- `/home/john/work/holon/wat-rs/src/rete/compiled_cond.rs:928: crate::rete::kernel::census_count("compiled:calls");` — real call. ✅
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/delta.rs:78: census_count("compiled:calls");` inside `77: let matched = if skip_span {` … `79: Some((0u32, 0u16))` — **the executor is not invoked; this is the elided-call fast path.**

**Verdict:** the surviving key counts "candidate (fact, alpha) pairs considered", of which one arm executes and one does not. The declaration calls it a *call* counter; a site that exists precisely to *skip* the call bumps it. Unit is "match attempts", not "calls".

## C. `GATHER_VISITS` / `census_gather_visit` — "elements examined by a gather" excludes most of the scanning paths

**Name/doc:** `/home/john/work/holon/wat-rs/src/rete/kernel/census.rs:218-233`, esp. 231 (*"Elements examined by an Accumulate/Negation/Exists gather"*) and 226-227 (*"if the gather still scans, the count still scales with the token count"*).

**Counted (4+2 sites):** acc.rs:346, 367, 379, 400; fire/mod.rs:1932, 1982; accumulate.rs:239.

**Uncounted scans of the same buckets:**
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/acc.rs:339: AccFold::Count => fold_i64s(fold, std::iter::empty(), bucket.len()),`
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/acc.rs:407: let gathered: Vec<&Element> = bucket.iter().map(|&i| &elements[i]).collect();` (Distinct/All/GroupBy/User)
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/accumulate.rs:252: gathered.extend(bucket.iter().map(|&i| &from_elements[i]));` (non-leftover arm)
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/mod.rs:1929: return !bucket.is_empty();` and `1960-1977` (`bucket.iter().map(...)` whole-bucket materialisation) — both no-`SeedCmp` arms.

**Verdict:** the quantity is "elements re-checked by a leftover predicate or an i64 fold", not "elements examined". The keyed-gather gate `keyed_gather_visits_do_not_scale_with_group_count` (`/home/john/work/holon/wat-rs/src/rete/kernel/tests/rank_and_instrument.rs:294-318`) rests on the broader claim; a regression to whole-memory scanning on any of the four paths above would register **zero** visits and the gate would stay green.

## D. `match:key-alloc` — a `match:` key that also accrues on the production-RHS path

**Name/doc:** `/home/john/work/holon/wat-rs/src/rete/eval_insert.rs:151-152` (*"NOT counted by `match:key-alloc`, which arms only the two resolve_operand sites"*) and `/home/john/work/holon/wat-rs/src/rete/kernel/tests/alpha_discrimination.rs:335-336` (*"armed at the two `Value::String(Arc::new(..))` call sites in `matcher.rs`"*).

**Sites:** `/home/john/work/holon/wat-rs/src/rete/matcher.rs:679` (Bind arm) and `/home/john/work/holon/wat-rs/src/rete/matcher.rs:930: crate::rete::kernel::census_count("match:key-alloc");` inside `resolve_operand`.

**Other callers of `resolve_operand` — not the matcher:**
- `/home/john/work/holon/wat-rs/src/rete/eval_insert.rs:287: if let Some(v) = resolve_operand(arg, &[], &[], bindings, None) {` (`resolve_rhs_value`, one per `?var` per derived fact)
- `/home/john/work/holon/wat-rs/src/rete/step_payload.rs:45: let Some(v) = resolve_operand(operand, fact_fields, field_names, bindings, Some(sym)) else {`

**Verdict:** the population is "binding-key `String` allocations anywhere `resolve_operand` runs", including `:then` RHS resolution and step-payload rendering. Latent, not live: `alpha_discrimination.rs:361/393` arms the census around direct matcher calls only. Any future whole-fire read of `match:key-alloc` gets RHS allocations folded into an alpha-match number.

## E. `match:calls` — counts calls that had a pattern, not calls

**Site:** `/home/john/work/holon/wat-rs/src/rete/matcher.rs:544-545`
```
544:     let pat = alpha_pattern(cond)?;
545:     crate::rete::kernel::census_count("match:calls");
```
**Verdict:** the `?` returns before the counter, so an invocation on a non-alpha `cond` is invisible. Name "calls"; quantity "calls that got past pattern extraction". Doc at matcher.rs:531-533 calls it the counter `compiled_cond.rs` parallels — but compiled_cond.rs:928 bumps *before* any early exit, so the two "call" counters are not the same unit.

## F. `dbeta:alloc` — an allocation name carrying a non-empty-result flag, consumed as a gather count

**Sites** (`/home/john/work/holon/wat-rs/src/rete/kernel/fire/mod.rs`):
```
970:        census_count_n("dbeta:calls", 1);
971:        census_count_n("dbeta:tokens", out.len() as u64);
972:        census_count_n("dbeta:alloc", u64::from(!out.is_empty()));
973:        census_count_n("dbeta:multi", u64::from(contributing > 1));
```
**Verdict:** `dbeta:alloc` is `count of calls whose result was non-empty` (0 or 1 per call), not a count of allocations — a `Vec` grown by several `extend`s allocates more than once, and the key can never exceed `dbeta:calls`. The consumer already renames it in prose: `/home/john/work/holon/wat-rs/src/rete/kernel/tests/node_share_cost.rs:287: let fire_gathers = counted("dbeta:alloc");` and the assertion at node_share_cost.rs:312-318 reads *"reported {fire_gathers} non-empty gathers"*. `dbeta:calls`/`dbeta:tokens`/`dbeta:multi` are correct.

## G. `prod:vec-alloc` / `prod:record-alloc` — a hardcoded ×2 per call wearing an allocation name

**Sites** (`/home/john/work/holon/wat-rs/src/rete/eval_insert.rs`):
```
187:    crate::rete::kernel::census_count_n("prod:vec-alloc", 2); // value_asts + fields
206:    crate::rete::kernel::census_count_n("prod:record-alloc", 2); // AggregateValue + the fields Arc
```
**Verdict:** neither is measured. `prod:vec-alloc` is `2 × calls`, but the kwargs branch of `rete_kwargs_value_asts` allocates **three** vecs (`/home/john/work/holon/wat-rs/src/rete/eval_insert.rs:55: let mut placed: Vec<(usize, &'a WatAST)> = Vec::with_capacity(args.len() / 2);` plus the result vec plus `fields`), the positional branch (eval_insert.rs:46) two, and `Vec::with_capacity(0)` at eval_insert.rs:188 allocates none. The name says allocations; the quantity is a constant multiple of calls.

## H. `merge:pv-owners` — a gauge summed into a counter, with a sentinel 0

**Site:** `/home/john/work/holon/wat-rs/src/rete/kernel/fire/rules.rs:807: census_count_n("merge:pv-owners", pv.array_owners() as u64);` (`array_owners` = `Arc::strong_count`, `/home/john/work/holon/wat-rs/src/value/pvec.rs:55-60`, returning **0** for the `Tree` arm).

**Verdict:** name reads as a count of owners; the accumulation is Σ-over-calls of a strong count, i.e. only meaningful divided by `merge:pv-calls` (rules.rs:808), and a 0 means "Tree representation", not "zero owners". Mitigated — but only in the *consumer*: `/home/john/work/holon/wat-rs/src/rete/kernel/tests/strat_cost.rs:475-477` and 489-495 spell both facts out. Nothing at the increment site says it.

## I. `seed:mixed-class-activate` — class-shaped name, per-FACT increment

- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/alpha.rs:188: census_count("seed:batch-class-mixed");` — per class ✅
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/alpha.rs:194: census_count("seed:batch-class-uniform");` — per class ✅
- `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/alpha.rs:286: census_count("seed:mixed-class-activate");` — **inside `for (i, fact) in input_facts.iter()`, i.e. per FACT**

**Verdict:** benign but real unit skew inside one family — two keys count classes, the third counts facts. The consumer knows (`/home/john/work/holon/wat-rs/src/rete/kernel/tests/pass_semantics.rs:751-753` asserts `activated == 3` "fact(s)" against `mixed == 1` "class(es)").

## J. `TestSummary.total` — total tests, but `failed` also counts per-FILE errors

**Declaration:** `/home/john/work/holon/wat-rs/src/test_runner.rs:78-80` (`pub total / passed / failed: usize`).
**Sites:** `260: summary.total += discovered.len();` (tests) vs `189: summary.failed += 1;` (directory read error), `219: summary.failed += 1;` (file read error), `255: summary.failed += 1;` (freeze error), and the genuine per-test `301/308/319/333`.
**Verdict:** `passed + failed` can exceed `total`; `270: println!("running {} tests", summary.total);` then `354-355: "test result: {}. {} passed; {} failed"` reports more results than tests announced. Name says tests; `failed` accumulates tests + file-level errors.

## K/L/M — smaller, doc-vs-quantity

- **census.rs:306-312 and 566-572** justify the `PHASE_NANOS` pair column with *"the `alpha:*` marks fire PER FACT … THREE of alpha's five children (candidates/element/fieldnames)"*. Only two `alpha:*` marks exist now — `/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/alpha.rs:232` and `:342` — both **once per pass**, and the three named children are gone. The pair count itself is correct (`census.rs:338: e.1 += 1;`); its stated magnitude is stale.
- **`right_idx_appended` doc `/home/john/work/holon/wat-rs/src/rete/kernel/census.rs:527-529`**: *"Called with `n == 0` too: 'the block ran and appended nothing' and 'the block never ran' are different facts."* True only for step 2 (`/home/john/work/holon/wat-rs/src/rete/kernel/fire/pass/hash_join.rs:340-344`). The maintainer site is inside `if already < right_elements.len()` (`/home/john/work/holon/wat-rs/src/rete/kernel/fire/mod.rs:810` → `:833`) and catch-up inside `if first_keying` (`hash_join.rs:161` → `:213`), so neither can ever emit a 0 row — the exact blind spot the doc claims is closed.
- **Benchmark replica bumps production keys**: `/home/john/work/holon/wat-rs/src/rete/kernel/tests/node_share_cost.rs:475-476` and `:500-501` call `super::census_count("filter:test-reuse")` / `("filter:test-pass")` from a synthetic timing loop that evaluates no predicate and pushes no token. Currently harmless (they sit outside the `with_count_census` window opened at node_share_cost.rs:273), but they are live writes to a production key from a bench replica, repeated per timing sample.

---

**Clean — checked and no mismatch found:** `match:head-miss`, `match:clause`, `match:bind-insert`, `rematch:compiled` (compiled_cond.rs:1129), `filter:test-evals`, `filter:test-reuse`, `filter:test-env-builds`, `filter:test-key-alloc`, `prod:class-alloc`, `prod:derivations`, `accum:index-builds`, `accum:index-elements`, `alpha:leaf-fill-pairs`, `elem-card:*`/`tok-card:*`/`bind-card:*` (delta.rs:769-776), `dbeta:calls`/`tokens`/`multi`, `beta_written`/`beta_read` (pass/mod.rs:46, 67, 151; hash_join.rs:124, 178), all `RoundCensus` fields, `ARM_BUILDS` (arm.rs:908 — once per `build_rete_arm`; note the rune comment at arm.rs:727 calls it an "intern-miss count" while a direct build at `where_tree_branch_differential.rs:174` also bumps it), and `alloc_counter.rs` `bump` (bytes in, bytes out, delta-correct `realloc`).
