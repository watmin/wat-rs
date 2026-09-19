# Solvere Ward Report — src/rete/kernel/** (excl. tests) + wat/rete/oracle/**

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

## Scope covered
- `src/rete/kernel/{mod.rs, session.rs, outcome.rs, node.rs, insert.rs}` — 2631 lines
- `src/rete/kernel/{arm.rs, census.rs, stratify.rs}` — 3446 lines
- `src/rete/kernel/fire/{mod.rs, rules.rs, delta.rs, acc.rs}` — 4714 lines
- `src/rete/kernel/fire/pass/{mod,root_join,join_after_filter,round_census,production,filter,filter_after_join,alpha,hash_join}.rs` — 2344 lines
- `wat/rete/oracle/{accum-pass,explain,pass,insert,stratify,fire}.wat` — 2326 lines

All 28 target files read in full, read-only, no cargo run. Total ≈15.46k lines. Per the cast's instruction, the Rust fast path and the wat oracle are judged as a deliberately parallel dual-implementation design, NOT as a braid against each other; findings below are all braids *within* one implementation.

## Findings

### 1. Domain concepts misplaced — termination verifier lives inside `stratify.rs`
- **File:line**: `src/rete/kernel/stratify.rs:1-2` (module doc claims the file is stratum-numbering only) vs. `src/rete/kernel/stratify.rs:330-1067` (`TerminationProof`, `RuleEdge`, `rete_fn_body_mints`, `body_constructs_computed`, `then_form_computes`, `domain_cardinality`, `computed_fields_are_provably_finite`, `binds_var_from`, `fences`, `computed_head_is_monotone_bounded`, `TerminationVerdict`, `refuse_non_terminating`).
- **Braided concerns**: (A) stratum numbering for stratified negation (`native_stratify_sweep`/`native_stratify_fix`/`native_stratify`/`native_rule_stratum`, lines 219-328) — the file's stated purpose and dual of `wat/rete/oracle/stratify.wat`. (B) fixpoint-termination/halting analysis via cycle detection + two independent finiteness proofs — a categorically different question, never named in the file's header.
- **Where each should live**: (A) stays in `stratify.rs`. (B) belongs in its own module (e.g. `kernel/termination.rs`), importing the shared graph helpers (`fact_type_head`, `rule_produces`, `rule_negates`, `consume_types`) rather than living inside the stratum-numbering file.
- **Recommendation**: split (extract ~lines 437-1067 into a new module).
- **Judgment**: structural — 69% of the file implements an undocumented second concern, and the module doc's "dual of the oracle" claim is only true of the smaller half.

### 2. Duplicated encoding — constructor-desugar head list, held in two places by hand
- **File:line**: `src/rete/kernel/stratify.rs:532-544` (`body_constructs_computed`'s `is_constructor` check, hardcoding `":wat::core::kwargs-construct"` and `":wat::core::aggregate-new"`).
- **Braided concerns**: "which desugared head forms constructor sugar bottoms out in" is encoded here as literal string comparisons; per the function's own comment (lines 539-541) the same two heads are separately maintained in `purity.rs`'s `KNOWN_UNREVIEWED` list.
- **Where it should live**: one shared constant (e.g. `purity::DESUGARED_CONSTRUCTOR_HEADS`) read by both sites.
- **Recommendation**: introduce abstraction, or mark `rune:solvere(historical-shape)` if not worth the churn now (currently unmarked either way).
- **Judgment**: incidental — narrated and reasoned-about by the author, but unchecked; a drift here fails silently. (Note: `purity.rs` itself is outside this cast's target scope, so this finding rests on `stratify.rs`'s own comment as citation.)

### 3. Duplicated encoding — reverse-children graph inversion, written twice
- **File:line**: `src/rete/kernel/fire/rules.rs:80-96` (in `fire_rules_stratified`) and `src/rete/kernel/fire/rules.rs:522-530` (in `requery_closed_world`).
- **Braided concerns**: both independently invert forward `node_children()` edges into a `ParentsOf` map by hand-walking `sorted_node_ids(network)`.
- **Where it should live**: one `build_rev_children(network) -&gt; ParentsOf` helper called from both sites.
- **Recommendation**: split into a shared helper.
- **Judgment**: incidental — small, mechanical, in-file copy-paste, but the exact "duplicated encoding" shape the ward names: which edges/node-kinds count must be kept in sync by hand.

### 4. Duplicated encoding — query-closure computation run twice per non-monotonic requery
- **File:line**: `src/rete/kernel/fire/rules.rs:551-571` (`requery_closed_world`, closure computed only to answer the boolean gate `feeds_a_non_monotonic_node`) and `src/rete/kernel/fire/rules.rs:401-416` (`harvest_stratified_queries`, called at `rules.rs:583`, re-deriving the identical worklist-seed + `close_upstream` walk over the same `arm.kind_ids.query`/network).
- **Braided concerns**: "decide whether a requery is needed" and "compute the requery's closure" are separate concerns but both re-derive the same closure independently instead of one being handed the other's result.
- **Where it should live**: a single query-closure builder, computed once by the gate and passed into the harvest step (or split into compute-closure / harvest-given-closure).
- **Recommendation**: introduce shared abstraction.
- **Judgment**: incidental but real — the file's own performance commentary (lines 532-550) defends the double walk on cost grounds, not architecture; a future change to "what counts as upstream" has two independent sites to update in lockstep.

### 5. Duplicated encoding — HashJoin left-activation body, triplicated (one copy contradicts its own doc comment)
- **File:line**: `src/rete/kernel/fire/pass/mod.rs:94-139` (`left_activate_join`, the extracted helper) vs. inline duplicates at `src/rete/kernel/fire/pass/join_after_filter.rs:84-113` and `src/rete/kernel/fire/pass/filter_after_join.rs:208-243` (grandchild-chain branch).
- **Braided concerns**: "encode a keyed HashJoin left-activation" (build `FilterJoinIdx`/`FireCtx`, call `keyed_join_persistent`, `record_tokens`) is expressed three times with byte-for-byte identical struct-literal shapes instead of once.
- **Verification**: `left_activate_join`'s own doc comment (`mod.rs:78-93`) claims the extraction fixed two duplicate sites plus a third new one; `grep -rn left_activate_join src/rete/kernel/` shows it is called from exactly one place (`filter_after_join.rs:75`, the *third*/new site) — the two original duplicates the comment says were fixed are still present, unconverted. The doc comment's claim is false against the code as it stands.
- **Where it should live**: `join_after_filter.rs:84-113` and `filter_after_join.rs:208-243` should call `left_activate_join`, deleting the inline copies; the doc comment should be corrected or the migration finished.
- **Recommendation**: split/replace call sites; fix the doc comment.
- **Judgment**: structural — fire-hot-path join logic (key computation, index writer) that must stay in lockstep; the team intended one source of truth and only partially delivered it.

### 6. Duplicated encoding — Negation/Exists dispatch, triplicated across two files
- **File:line**: `src/rete/kernel/fire/pass/filter.rs:231-288` (else-arm of `filter_pass`'s dispatch), `src/rete/kernel/fire/pass/filter_after_join.rs:120-166` (`hj_id`-level branch), `src/rete/kernel/fire/pass/filter_after_join.rs:247-297` (grandchild `chain` walk, same branch again).
- **Braided concerns**: "resolve pass/fail for a Negation or Exists node" (hoist join keys via `gather_join_keys`, loop `token_exists_under`, invert verdict by kind) is near-verbatim identical in all three (compare `filter.rs:237-256`, `filter_after_join.rs:130-149`, `filter_after_join.rs:256-275`).
- **Where it should live**: one shared helper beside `left_activate_join` in `pass/mod.rs`, parameterized over token-source ownership and the `leading_emitted` dedup that only `filter.rs`'s "leading" case needs (`filter.rs:277-284`) — the one genuine divergence.
- **Recommendation**: introduce abstraction.
- **Judgment**: structural — same class of correctness-sensitive fire-hot-path logic as Finding 5; the codebase already knows the pattern to apply (it applied it for HashJoin) but didn't apply it here.

### 7. Duplicated encoding (wat oracle) — "unwrap-a-must-succeed-Outcome" protocol repeated ~7×
- **File:line**: `wat/rete/oracle/fire.wat:276-285` (`fire-grow-fixpoint`), `fire.wat:347-356` (`fire-support-fixpoint`), `fire.wat:509-518` (`fire-stratified`), `wat/rete/oracle/explain.wat:66-75` and `explain.wat:92-111` (`fire-rules-explain$oracle`, two unwraps), plus a sibling `InsertOutcome` shape at `wat/rete/oracle/insert.wat:52-57` and `fire.wat:446-451`.
- **Braided concerns**: calling the oracle primitive (legitimate per-site) vs. the fixed 3-arm "only one arm is reachable, panic on the others" protocol, hand-re-derived at every call site instead of expressed once.
- **Where it should live**: one shared helper each, e.g. `:wat::rete::expect-fired [outcome ctx] -&gt; Session` and `:wat::rete::expect-inserted [outcome ctx] -&gt; Session`.
- **Recommendation**: introduce abstraction (factor after the documented bootstrap-ordering constraint that blocked the *first* instance no longer applies to a *later* factoring pass).
- **Judgment**: incidental — grew from independent application of a one-time bootstrap pattern (arc 278 "fire-outcome wall") at each new call site.

### 8. Duplicated encoding (wat oracle) — "derive facts for a token via RHS" traversal, written twice
- **File:line**: `wat/rete/oracle/pass.wat:459-502` (`fire-production`, inner fold at 480-497) vs. `wat/rete/oracle/explain.wat:10-49` (`harvest-support`, inner fold at 28-44).
- **Braided concerns**: both independently walk a ProductionNode's parent tokens and RHS insert-forms via `eval-insert`, then diverge only in what they do with the derived fact (conj into `prod-mem` vs. build a first-producer-wins support map).
- **Where it should live**: a shared `derived-facts-for-token [rhs tok] -&gt; PersistentVector&lt;Record&gt;` helper folded over by both callers.
- **Recommendation**: introduce abstraction / split.
- **Judgment**: incidental — `explain.wat` is a later addition ("loads after fire.wat") re-deriving logic that already existed in `pass.wat`.

### 9. Justified braid missing its formal rune (wat oracle) — accumulator dispatch tail repeated 6×
- **File:line**: `wat/rete/oracle/accum-pass.wat:28-142` (`accumulate-pass-for-token`) — the `count`/`sum`/`distinct`/`all`/`group-by`/custom-fold branches (lines 46-49, 56-59, 102-105, 108-111, 118-121, 140-142) each repeat the identical `PersistentMap/assoc` + `Token` build + `append-token` tail.
- **Braided concerns**: "which fold to run" and "how to store the result" are re-derived together per branch instead of factoring storage out once.
- **Justification present but unstamped**: the file's own comment (lines 11-16, 207-209) explains this is forced by wat's invariant parametric types (`Option&lt;i64&gt;` is not a subtype of `Option&lt;Value&gt;`), so a single typed dispatch point cannot be written — a real `load-bearing-coupling` justification, but expressed only as prose, not as a rune.
- **Recommendation**: mark with rune — add `;; rune:solvere(load-bearing-coupling) — wat's invariant parametric types make (Option :- [i64]) ≠ (Option :- [Value]), so each fold's concrete return type must be handled inline; a single dispatch point cannot be typed` above the `defn` at `accum-pass.wat:28`.
- **Judgment**: structural but a paperwork gap, not a design defect — the constraint is real and already documented, just not formalized as the rune the ward asks for.

## CONVERGED (no findings)
- `src/rete/kernel/mod.rs`, `session.rs`, `outcome.rs`, `node.rs`, `insert.rs` — read in full; the three near-identical `Outcome` converters in `outcome.rs` are three distinct closed-ceiling-set concerns (exhaustive matches, no wildcard), not one duplicated; `session.rs`'s size is walled into declared sub-concerns with private single-write-door indices; the ceiling check centralized in `session.rs` is the documented cure for a prior braid, not a new one.
- `src/rete/kernel/arm.rs` — orchestration by design, each phase a named helper call; no misplaced concept or duplicated encoding.
- `src/rete/kernel/census.rs` and `src/rete/kernel/fire/pass/round_census.rs` — entirely `#[cfg(test)]` instrumentation, the ward's own dedicated-channel exemption.
- `src/rete/kernel/fire/pass/{root_join,production,alpha}.rs` — `alpha.rs`'s `ClassPlan` is a genuinely insulating wrapper; its three `AlphaActivateCx` literals all call the one shared `alpha_activate_fact`; `production.rs`'s support-threading is one job, one branch.
- `src/rete/kernel/fire/pass/hash_join.rs`'s `hj_step3_term1`/`hj_step4_term2` vs. the catch-up cross-join block — structurally close but operate over different token universes (full `all_left` vs. delta `dl`/`dr`) with a documented, checkable reason they aren't merged; judged borderline and not flagged.
- `src/rete/kernel/fire/{mod,delta,acc}.rs` — `FireCtx`/`GatherIntern::from_wm`/`from_ctx` pairs are borrow-checker-driven glue delegating to the same real logic (no drift risk), already documented as such in `mod.rs:84-102`; census/phase-timing calls are a dedicated cross-cutting channel; the large `fire/mod.rs` groups by firing-phase, not by unrelated domain; `SeenSet` (delta.rs) vs. `facts_membership`/`merge_facts` (rules.rs) solve different dedup problems via the generic `HashSet` idiom, not shared domain encoding.
- `wat/rete/oracle/fire.wat`'s four `walk-*-ids` TCO walkers (documented deliberate split from one polymorphic walker) and `pass.wat`'s single `topological-node-ids` definition (documented, verified as the only sort definition, called by both `fire.wat` and `explain.wat`) — clean, intentional decomplection, not findings.
- The repeated `match (Map/get m k) (Some v → …) (None → default)` idiom (~8+ sites across `pass.wat`/`fire.wat`) — mechanical boilerplate with no drift risk, out of solvere's remit.

## Runes encountered (all families, per instruction to record every rune found)

No `rune:solvere(...)` marks exist anywhere in the 28 target files. All runes found belong to other wards; recorded for completeness, not adjudicated under solvere:

- `rune:struere(...)` — `node.rs:83`; `session.rs:20,61,85,97,109,616,1686,1716`; `mod.rs:1525`, `acc.rs:81,157,201,214` (invariant/lifetime-coupling); `acc.rs:13` (host-constraint) — all in `fire/{mod,acc}.rs` and `session.rs`/`node.rs`.
- `rune:excusare(no-falsifier)` — `session.rs:1834`.
- `rune:sequi(performance-counter)` / `rune:sequi(ambient-context)` — `census.rs` (multiple), `arm.rs:705,727`.
- `rune:perspicere(read-once)` / `rune:perspicere(intentional-structure)` — `census.rs` (multiple), `arm.rs:1072`, `wat/rete/oracle/pass.wat:245`.
- `rune:circumspicere(accepted-by-design)` — `arm.rs:717`.
- `rune:temperare(simplicity-win)` — `fire/mod.rs:311,494`, `fire/rules.rs:682`.
- `rune:lint(gather-walk-not-examining)` — `fire/mod.rs:840,948`.
- `rune:lint(cited-name-absent)` — `fire/rules.rs:167`, `fire/acc.rs:328`.
- `rune:intueri(naming)` — `wat/rete/oracle/fire.wat:54-56`.
- Non-rune prose mention only: `stratify.rs:350` (mentions "rune:" inside a rejected-proposal comment, not an actual marker).
- Informal (non-formatted) solvere-motivated note, not a rune tag: `fire/pass/hash_join.rs:563` — a comment documenting a past move of `seed_dirty_join_parents` out of `arm.rs` for being misplaced fire-round logic; evidence the discipline has been applied once here and, per Finding 5, incompletely elsewhere.

All non-solvere runes reviewed carry non-empty, substantive reasons for their own wards; none required a solvere verdict since none claim a solvere exemption.
