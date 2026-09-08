# COMPLECTENS — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## 1. What I swept

**Inventory re-derivation (all confirmed exact, zero deltas from the brief):**
```
find tests/rete src/rete/kernel/tests -type f | wc -l                          → 306
find tests/rete src/rete/kernel/tests -type f -exec cat {} + | wc -l           → 38058
tests/rete/: 144 .wat, 100 .rs, 23 .edn, 19 .wat.bad = 286
src/rete/kernel/tests/: 20 .rs
grep -rhE '^\s*#\[test\]' tests/rete src/rete/kernel/tests | wc -l             → 613
```
All thirteen numbers in the brief held. No deltas to report.

**Phase 1 — mechanical, reproducible, ran to completion:**
- `.rs`: a Python paren/brace-balanced extractor over every `#[test] fn …{…}` (handles both the two-line and one-liner `#[test] fn f() { … }` forms) across both trees → **613/613 tests found**, sorted by body-line-count descending.
- `.wat`: same technique keyed on `(:wat::test::deftest …)`, paren-balanced → **18 wat-level deftests found**.
- Cross-check: only **5 of 144 `.wat` files** contain any `:wat::test::deftest` form at all; the other **139** carry only `defn` fixtures with no wat-level test macro. Verified this isn't a scan miss: 95 of the 100 `.rs` files call `startup_beside(file!())`, confirming the dominant architecture is *co-located fixture* (`.wat` = world/rule data, `.rs` = the `#[test]` + assertions), not standalone wat test files.

**Phase 2 — judgment, sampling rule stated:** I read, in full, every `.rs` test body ≥150 lines (10 files/13 tests) plus every one of the 5 `.wat` files carrying real `deftest` forms (all 18, all tiny — max 36 lines, none flagged). For files identified as "large" by line count (six 1100+-line files, `probe_arc278_export.rs`), I checked the file's per-test body-line distribution before treating the file size as a signal, per the brief's own caution. I additionally read `src/rete/kernel/tests/mod.rs` in full (697 lines) to check top-down helper ordering, and spot-checked the `probe_arc278_D10/D11` pair and 3 `probe_arc278_4a/4b/4c` files for the stepping-stone question. I did **not** read the remaining ~590 `#[test]` bodies under 100 lines individually.

## 2. Phase 1 survey — head of the sorted list

```
718  src/rete/kernel/tests/node_share_cost.rs:100   node_share_where_cost_decomposition   [RUNE: inline-fixtures]
301  src/rete/kernel/tests/accum_alpha_cost.rs:74   accum_alpha_leftover_split
291  src/rete/kernel/tests/accum_alpha_cost.rs:973  accum_alpha_push_split
285  src/rete/kernel/tests/accum_alpha_cost.rs:382  accum_alpha_seed_after_fold_split
257  src/rete/kernel/tests/gather_probe_cost.rs:924 probe_gap_cost_split
256  src/rete/kernel/tests/accum_cost.rs:1051       accum_materialize_split
224  src/rete/kernel/tests/accum_cost.rs:821        accum_compiled_match_split
211  src/rete/kernel/tests/accum_cost.rs:605        accum_leftover_split
200  src/rete/kernel/tests/fanout_cost.rs:471       fanout_three_leftover_split
177  src/rete/kernel/tests/gather_probe_cost.rs:741 probe_extend_cost_split
176  src/rete/kernel/tests/alpha_discrimination.rs:352  compiled_cond_failure_path_allocates_no_binding_keys_at_50_100
169  src/rete/kernel/tests/accum_alpha_cost.rs:798  accum_alpha_class_lookup_split
169  tests/rete/wat_scripts_grid_port_check.rs:339   every_grid_axis_native_matches_its_oracle
165  src/rete/kernel/tests/accum_cost.rs:1313        accum_intern_val_i64_split
162  src/rete/kernel/tests/accum_cost.rs:437         accum_harvest_index_parts
158  src/rete/kernel/tests/gather_probe_cost.rs:447  gather_unary_index_split
154  src/rete/kernel/tests/accum_cost.rs:276         accum_query_harvest_split
150  tests/rete/wat_scripts_grid_axes_live.rs:387    grid_axes_run_and_derive_nonvacuously
145  src/rete/kernel/tests/accum_cost.rs:1484        accum_exec_ops_split
143  src/rete/kernel/tests/cascade_cost.rs:182       cascade_query_harvest_split
114  src/rete/kernel/tests/pass_semantics.rs:99      guiding_light_matches_carry_support_chain  [RUNE: proof-stepping-stones]
```

`.wat`, all 18, complete (max is 36 lines — none candidate-flagged): `probe_arc278_sqlite_interop.wat:12` 36; `probe_arc278_sqlite_store_differential.wat:83` 27; `probe_arc278_cache_lru.wat:13` 14; remaining 15 all ≤12 lines.

**Large-file-vs-large-test check** (the trap the brief flagged): `probe_arc278_export.rs` is 1199 lines with **31 tests**, max body **60 lines** (`empty_deps_import_refuses_fire:176`), median well under 20. A large file, not a large test — confirmed by reading its full test list.

## 3. Findings

**Finding A (Level 3 — consistency/documentation gap).** `src/rete/kernel/tests/node_share_cost.rs:98-99` carries `rune:complectens(inline-fixtures)` justifying its 718-line, ~23-outer-binding interleaved-timing test. At least **7 sibling files carry the structurally identical idiom at comparable or larger scale, with no rune at all**: `accum_alpha_cost.rs` (301/291/285/169-line tests, confirmed 23 outer bindings on `accum_alpha_leftover_split:74`), `accum_cost.rs` (256/224/211/165/162/154/145-line tests), `gather_probe_cost.rs` (257/177/158-line tests, read `probe_gap_cost_split:924` in full — same shape: named phase variables `r,s,p,e,j,g`, per-component non-vacuity asserts naming the component), `fanout_cost.rs`, `cascade_cost.rs`, `harvest_cost.rs`, `strat_cost.rs`. I read three of these bodies end-to-end and applied the four questions: **Honest** (names match bodies), **Obvious** (every phase-variable has its own named non-vacuity assert, e.g. `"alpha:seed recorded 0 — the mark never fired"`), **Good UX** (top-down, no forward refs). Judgment: **not a Level 1 violation** — same legitimate "inline-fixtures" shape as the one exempted file — but the exemption documentation was applied to one file and not its near-identical siblings, which is a real inconsistency worth naming, since a future reader hitting one of the un-rune'd files has no signal that its shape was already adjudicated elsewhere.

**Finding B (Level 2 — helper with only a partial proof).** `src/rete/kernel/tests/mod.rs:483-594` defines `render_phase_table`, a shared, non-trivial arithmetic helper (instrument-subtraction: `net_of`, `total_min`, `total_net`, per-phase percentage, a "BELOW ITS OWN INSTRUMENT" flag) used at 4 call sites (`fanout_cost.rs:355`, `accum_cost.rs:219`, `node_share_cost.rs:961`, `rank_and_instrument.rs:263`). Its own doc comment (mod.rs:502-505) records **two past arithmetic bugs**: it "returned `sum/xs.len()` [a mean] until arc 278 C1" under a MINIMUM-labeled header, and "an earlier version of this table report 124% coverage" from double-counting parent+child rows. There **is** one dedicated test, `rank_and_instrument.rs:260-271` `render_phase_table_proves_missing_phase_and_zero_total` — but I read it and it feeds a census missing the one `required` phase, which hits the `missing.is_empty()` panic **before** the function reaches the arithmetic section at all. So the one function-level proof that exists covers only the early-exit guard; the numerically load-bearing part — exactly the part with the documented history of being wrong — has zero direct proof and is exercised only by eyeball via `println!("{table}")` in its 4 callers, none of which assert on the rendered net/percentage values (I confirmed `node_share_cost.rs:970-996` explicitly asserts only on raw `ns_of(...)` values pulled independently of `table`, with a comment "Assert on the DATA, not the rendered text"). Direction: add a case that drives the arithmetic path with fixed synthetic census rows and asserts a known `net`/percentage, closing exactly the gap its own history describes.

**Finding C (Level 3 — minor, proof-need is arguable).** `src/rete/kernel/tests/alpha_discrimination.rs:38-49`, `fn alpha_tree_fixture_50_100() -> AlphaTreeFixture`, composes 5 non-trivial steps (`fire_cascade` → `to_transient` → `sorted_node_ids` → `build_alpha_index` → `AlphaTree::build`) and is consumed by 4 tests (lines 67, 139, 244, 362) with no dedicated test of its own. Each `.expect(...)` inside it carries a distinct message, so a break does narrow somewhat even without a sibling deftest — I judge this borderline-acceptable (proof-need axis: it composes already-tested library primitives rather than introducing new logic), but flagging it since it's the one helper-without-sibling-test candidate the mechanical pass turned up outside the cost-benchmark family.

**Non-finding worth recording:** `tests/rete/wat_scripts_grid_axes_live.rs:387` (`grid_axes_run_and_derive_nonvacuously`) and its sibling `wat_scripts_grid_port_check.rs:339` are large (150/169 lines) but are themselves **exemplars of the exact vacuity discipline** the brief's closed-class finding is about — I read `grid_axes_run_and_derive_nonvacuously` in full: it asserts the *exact set* of on-disk axis stems against a hardcoded list specifically so "an empty glob or a moved `wat-scripts/perf/grid/`... fail[s] loudly instead of passing vacuously" (its own comment, line ~415). Not a new instance of the vacuity trap; a correctly-hardened one.

**`.wat.bad` / D10-D11 family:** `probe_arc278_D10_then_field_types.rs` and `_D11_nested_then_field_types.rs` each drive one shared `run()` helper across 5-6 named tests, one per `.wat`/`.wat.bad` variant (bound_var/literal/positional/computed) — read in full, this is a clean, correctly-composed single-file family, not a violation.

## 4. The `probe_arc278_` answer: **independent probes, not a stepping-stone family**

Evidence:
- **No cross-file code dependency exists or could exist.** `grep -l '^mod \|include!\|#\[path'` across all 257 `probe_arc278_*.rs` returns nothing — each is a separate cargo-integration-test binary (top-level file under `tests/`), so no `probe_arc278_2a_*.rs` can call into `probe_arc278_2b_*.rs`'s functions; Rust's own compilation model rules it out structurally.
- Read `probe_arc278_2a_alpha_match.rs`, `2b_insert_alpha.rs`, `4a_production_fire.rs`, `4b_cascade.rs`, `4c_retraction.rs` headers and bodies in full: each documents a **different** feature ("2a" = the alpha matcher in isolation; "2b" = insert+fire-once; "4a" = production firing; "4b" = cascade; "4c" = retraction), each is fully self-contained against its own co-located `.wat`, and cross-references to siblings are explicit **prose** pointers ("See `probe_arc278_alpha_is_fire_scoped.rs` for the differential covering the fixpoint verbs" — `2b_insert_alpha.rs:11`), never a shared function or shared state.
- Test bodies in this family are uniformly tiny (5-10 lines: one `call_beside_value(file!(), ...)` + one `assert_eq!`), the opposite shape of a stepping-stone family.
- The `1a/1b/2a/2b/.../8a/8b/8i/8custom` numbering is a chronological/architectural stage label for arc 278's engine build-out (each number a pipeline stage: data-model → compile → alpha-match → insert → ... → accumulate), not a decomposition of a single scenario.

**Verdict: 257 independent probes sharing a naming/numbering convention that documents build order, not a Level 2 stepping-stone violation.** This matches the brief's own hedge exactly.

## 5. The two runes

- `src/rete/kernel/tests/pass_semantics.rs:97` — `rune:complectens(proof-stepping-stones)`. Read the exempted test (`guiding_light_matches_carry_support_chain`, 99-215, 114-line body): it runs the four named production-code passes (`alpha_pass`, `root_join_pass`, `hash_join_pass`, `production_pass` — already-tested kernel functions, not test helpers) then a sequence of assertions on the resulting token structure. **Reason reads sound** — there is no reusable sub-scenario to extract; the point is exercising exactly this pipeline order.
- `src/rete/kernel/tests/node_share_cost.rs:98-99` — `rune:complectens(inline-fixtures)`. Read the full 718-line body: extensive doc-comment justification above it (explicit ABAB-drift-immunity requirement, citing a specific past incident). **Reason reads sound**, and — per Finding A — the same reasoning legitimately covers several un-rune'd siblings that were not exempted.

Both skipped per the spell's instruction; not re-adjudicated beyond the soundness note requested.

## 6. What I looked for and did not find

- No forward-reference violations in `src/rete/kernel/tests/mod.rs` (697 lines, read in full): general helpers (54-208) precede the mod declarations that need only those (221-224); benchmark-specific helpers (413-680) precede the mod declarations that consume them (683-697). Top-down holds.
- No additional `rune:complectens` beyond the two named in the brief (grepped the full target).
- Did not find a second instance of the vacuity-gate class-cure gap — the two large "grid" tests I read are hardened exemplars of it, not new instances.
- Did not apply the four questions individually to the ~590 tests under 100 lines (stated sampling boundary above).
- Did not find any `.wat` file among the 139 fixture-only files that itself needs a rune — they carry no `deftest`, so the mechanical trigger for this ward's line-count/binding-count checks does not apply to them; their correctness is proven by the sibling `.rs` file's assertions, which is the corpus's stated architecture, not a violation of it.

**FINDINGS**
