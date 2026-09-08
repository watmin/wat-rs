## PERSPICERE — Cast Report

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> Status for every row lives in `FINDINGS.md`, and nowhere else.
> HTML entities (`&lt;` `&gt;` `&amp;`) are artifacts of the agent's own output encoding.

## SCOPE

**Files read in full** (all 22 in-scope `.rs` files, plus the 6 oracle `.wat` files):
`src/rete/kernel/{arm,census,insert,mod,node,outcome,session,stratify}.rs`, `src/rete/kernel/fire/{acc,delta,mod,rules}.rs`, `src/rete/kernel/fire/pass/{accumulate,alpha,filter,filter_after_join,hash_join,join_after_filter,mod,production,root_join,round_census}.rs`, `wat/rete/oracle/{accum-pass,explain,fire,insert,pass,stratify}.wat`. `src/rete/kernel/tests/**` was not opened.

**Method — Rust side:** reproduced `grep -cE '<[^<>]*<'` over all 22 files. Result matched the given map exactly: same 14 files, same per-file counts (34/16/12/10/8/5/5/3/2/1/1/1/1/1 = 100 raw hits, 0 in the other 8 files — verified those 8 by hand, no multi-line generics hiding from the line-based regex). **No divergence on the raw count.** I then read every hit in its function/struct context and classified it: real nested-generic type (flagged), false positive (comparison operator, string literal, prose comment, or a turbofish `::<Vec<_>>` which is depth-1, not depth-2), or a typealias body (exempt per spell). Of the 100 raw hits: ~15 are false positives, ~19 are typealias-declaration bodies (mostly `session.rs`'s existing alias block, plus `arm.rs:395`, `census.rs:503`), 10 are the pre-existing runes, and the remainder are genuine flagged sites, which cluster into far fewer *distinct* type shapes because the same shape repeats across call sites — that clustering is the real finding, reported below by shape rather than by raw line.

**Method — `.wat` side:** confirmed the two named false positives (`accum-pass.wat:27` prose, `insert.wat:51` a bind arrow). Derived the bracket-nesting analog: counted lines with 2+ occurrences of the token `:- [`, then verified each candidate wasn't an artifact of unrelated adjacent brackets. `explain.wat`, `insert.wat`, `fire.wat` and `stratify.wat` carry only depth-1 nesting. `accum-pass.wat` and `pass.wat` do carry genuine depth-2 bracket nesting — reported below.

## Flagged types (Rust)

**1. `Result<Arc<InternedNetwork>, EvalBreak>`** — depth 2. `arm.rs:850,862,876` (3 identical sites, one file, unruned). Sibling alias: none yet, but `arm.rs:395`'s `UserFoldPrograms` shows the file's own convention. **Recommendation: mint.**

**2. `Option<Arc<InternedNetwork>>` / `Option<Arc<crate::rete::kernel::InternedNetwork>>`** — depth 2. `arm.rs:731`, `fire/delta.rs:275` (same shape, cross-module, one path-qualified). **Recommendation: mint** (e.g. `ArmedNetwork`).

**3. `impl Iterator<Item = Result<i64, EvalBreak>>`** — depth 2. `fire/acc.rs:249,279,301`. `Result<i64, EvalBreak>` alone (depth 1) recurs 6× in the same file with no alias anywhere in the crate. **Recommendation: mint** `type I64Result = Result<i64, EvalBreak>;`.

**4. `Result<Option<Value>, EvalBreak>`** — depth 2. `fire/acc.rs:303,337,429` plus `eval_insert.rs:289` (outside scope, same shape) — 4 total repeats. **Recommendation: mint.**

**5. `Result<Vec<Token>, EvalBreak>`** — depth 2. `fire/mod.rs:804,870`, `fire/pass/hash_join.rs:514`. **Recommendation: mint.**

**6. `Option<std::sync::Arc<[Value]>>`** — depth 2. `fire/pass/filter.rs:237`, `fire/pass/filter_after_join.rs:130,256` — 3 sites across 2 files, **identical local variable name `hoisted_keys` at all three**. **Recommendation: mint** — strongest single case in the file set.

**7. `Option<(&'a GatherIndex, Arc<[Value]>)>`** — depth 2. `fire/mod.rs:2093,2111`. **Recommendation: mint** a lifetime-generic alias.

**8. `Option<&Arc<[Value]>>`** — depth 2. `fire/mod.rs:478,2137` (both parameter `join_keys`). **Recommendation: mint or rune** — borderline.

**9. `Vec<(i64, Option<usize>, usize)>`** — depth 2. `census.rs:75` (struct field) and `session.rs:414` (`per_join_marks` return). Precedent: `census.rs:34` already has `type RightIdxPrefix = …`. **Recommendation: mint** the inner tuple.

**10. `Result<HashMap<String, i64>, EvalBreak>`** — depth 2. `stratify.rs:281,304`; bare `HashMap<String, i64>` appears 3 more times in the same file, and the oracle already calls this shape `type-strata` in prose (`stratify.wat:41`). **Recommendation: mint** `type TypeStrata = HashMap<String, i64>;`.

**11. `RefCell<Option<GatherKeyMap>>`** — depth 2. `census.rs:509`. **Not in the given rune list, and unruned.** Structurally identical to the 7 already-runed census TLS instruments but was missed. **This is a real gap**, not a new defect class. **Recommendation: rune** (`read-once`, true: `GatherKeyMap` appears exactly once) or flag to the author that it was skipped.

**Leave alone (checked, judged not worth aliasing):** `HashMap<i64, JoinKeyMap<Token>>` / `<Element>` (`session.rs:227,345` — inner noun already named at `:204`, surrounding structs carry extensive WHY-comments); `Option<&Vec<Token>>` (`session.rs:472`), `impl Iterator<Item = &Vec<Token>>` (484), `impl Iterator<Item = (&i64, &Vec<Token>)>` (528), `Vec<Option<I64Row>>` (586) — standard accessor idioms on already-named structs; `Result<HashMap<i64, CondDriver>, EvalBreak>` and its two siblings (`arm.rs:161,348,486`) — each occurs exactly once and names a distinct value type; `RefCell<FxHashMap<u64, InternEntry>>` (`arm.rs:722`); `HashMap<String, Vec<String>>` (`stratify.rs:972`) — recurs 7+ times elsewhere in the crate with no alias anywhere, which reads as settled judgment; several single-occurrence function-local shapes in `fire/mod.rs`, `join_after_filter.rs:40`, `production.rs:34`; `bind_only`/`cond_key_ids` (`fire/mod.rs:134-135`) — field names already carry the noun.

## Flagged types (`.wat` oracle)

Confirmed the cast's hint: `:- [ … :- [ … ] ]` is the real bracket-nesting analog of `<…<…>>`.

**12. `(:wat::core::PersistentMap :- [:wat::core::i64 (:wat::core::PersistentVector :- [:wat::rete::Element])])`** and its `Token`/`Record` siblings — depth 2, appearing unaliased **8+ times** in `pass.wat` (`53,55,144,146,148,337,339,341,691,693,695,803`) and 2× in `accum-pass.wat` (`34,35`).

**`wat/rete.wat`** (outside the scoped target, but the defining sibling module) *already* declares exactly these as named typealiases: `:wat::rete::AlphaMemory`, `:wat::rete::BetaMemory`, `:wat::rete::ProductionMemory`. The oracle files spell the noun out longhand at every one of those sites instead. This is not "mint a new alias" — **the alias already exists and simply isn't being used** at these call sites. **Recommendation: use the existing sibling alias** (a `wat-fix` codemod job per this repo's own doctrine, not a hand-edit — out of scope for this read-only cast).

**13.** `pass.wat:245` — already runed (`intentional-structure`), adjudicated below.

No other oracle file carries 2+-level bracket nesting.

## The 10 existing runes, adjudicated

| # | Site | Category | Claim checked | Verdict |
|---|------|----------|----------------|---------|
| 1 | `census.rs:108` → `FIRE_CENSUS: RefCell<Option<Vec<RoundCensus>>>` | read-once | grepped whole repo, **1 occurrence** | **Clear.** |
| 2 | `census.rs:130` → `WHERE_SAMPLE: RefCell<Option<Option<WhereSample>>>` | read-once | **1 occurrence** | **Clear.** |
| 3 | `census.rs:194` → `LEAF_OCC_DIFF: RefCell<Option<Vec<LeafOccDiff>>>` | read-once | **1 occurrence** | **Clear.** |
| 4 | `census.rs:630` → `PHASE_NANOS: RefCell<Option<HashMap<&'static str, (u64, u64)>>>` | read-once | **1 occurrence** | **Clear.** |
| 5 | `census.rs:676` → `CENSUS_COUNTS: RefCell<Option<HashMap<&'static str, u64>>>` | read-once | **1 occurrence** | **Clear.** |
| 6 | `census.rs:735` → `BETA_TRAFFIC: RefCell<Option<HashMap<i64, (u64, u64)>>>` | read-once | **1 occurrence** | **Clear.** |
| 7 | `census.rs:837` → `RIGHT_IDX_APPENDS: RefCell<Option<HashMap<(i64, &'static str), usize>>>` | read-once | **1 occurrence** | **Clear.** |

Per the cast's specific instruction — no block verdict on these seven despite the templated wording. **Each read-once claim is independently, factually true**: I checked every one of the 7 exact type strings against the whole repo and each is unique. The seven are seven distinct instrumentation shapes for seven distinct `thread_local!` test instruments — not one template stamped over a repeated type. The "alias would be a mumble" half of each reason is weaker/more subjective than the "read-once" half, but the load-bearing claim (uniqueness) holds in all seven, so all seven pass.

| 8 | `fire/pass/alpha.rs:284` | intentional-structure | "Arc vs owned Vec is the occupancy-share door" | **Clear.** `Arc<Vec<Element>>` is exactly the value type of the existing `AlphaMemory` alias, and this is the one call site that immediately `Arc::clone`s into multiple `aids` — the annotation makes the sharing visible where it matters. |
| 9 | `arm.rs:1072` → `HashMap<Vec<i64>, Vec<i64>>` | read-once | grepped whole repo: **exactly once**; `by_parents` never escapes `build_test_sibs` | **Clear.** |
| 10 | `wat/rete/oracle/pass.wat:245` | intentional-structure | "Option vs empty-PV is the no-alpha door" | **Clear.** The `match` in `alpha-els-for-cond` distinguishes `Some(pv)` from the `None`-mapped legacy-fallback arm — the `Option` carries the "was an alpha even minted" fact. This rune is also what revealed the alias-reuse gap (#12). |

## CONVERGED

I did not find defects the ward is silent on — every raw regex hit was read and classified, every rune was individually checked against a whole-repo grep rather than trusted on its prose, and the `.wat` half was given its own derived detector rather than skipped. The result is not "clean": there are 13 genuine findings (11 Rust mint/rune candidates, one unruned gap at `census.rs:509`, one unused-sibling-alias gap spanning 10 oracle sites) plus a full, individually-verified adjudication of the 10 existing runes (all 10 clear). Severity throughout is **L2** — nothing here is an L1 correctness lie. Counts: 100 raw `<`-nesting hits reproduced exactly against the given crude map; 100 raw `:- [`-nesting occurrences swept across 6 oracle files, of which only `pass.wat` and `accum-pass.wat` carry genuine depth-2 nesting.
