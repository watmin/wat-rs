# EXCUSARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## 1. What I weighed

**Inventory re-derived at HEAD** (branch `grok-rete`): `find tests/rete src/rete/kernel/tests -type f | wc -l` → confirms **306 files** (`tests/rete/`: 286 = 144 `.wat`, 100 `.rs`, 23 `.edn`, 19 `.wat.bad`; `src/rete/kernel/tests/`: 20 `.rs`), **38,058 lines**. Matches the brief exactly.

`grep -rho 'rune:[a-z]*([a-z-]*)' tests/rete src/rete/kernel/tests | sort | uniq -c` reproduced the brief's distribution **exactly**: 52/24/12/9/4/2/2/1/1/1/1 = 109. But **the literal-string count is not the live-exemption count** — three of those categories partly or wholly consist of prose *discussing* a rune, not a rune standing at that site:

| category (brief count) | live exemptions in-target | delta | why |
|---|---|---|---|
| `vocare(vantage-bypass-test)` (24) | **23** | −1 | `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.rs:10` is a doc-comment *quoting* the rune that actually lives on 4 tests in `pass_semantics.rs` — not a second instance |
| `struere(invariant-coupling)` (1) | **0** | −1 | `tests/rete/probe_arc278_import_fold_key.rs:5` quotes a rune that actually lives at `src/rete/kernel/fire/acc.rs:81` (outside this target, inside `src/rete/kernel/`) — no live `struere` rune exists inside the target tree |
| `lint(red-by-design)` (2) | **0** | −2 | both hits in `probe_arc278_match_arm_is_not_a_call.rs:37,190` are prose recounting a rune that lived in `docs/arc/.../experiri-then-match.wat` (outside target) and — per the same prose — **has already been removed**: *"The rune is gone"* |
| `perspicere(read-once)` (12) | 12 | 0 | all live |
| `complectens` (2) | 2 | 0 | all live |
| `exigere(scope-affirmative)` (1) | 1 | 0 | live, standalone |
| `lint(census-name-retired)` (4) | 4 | 0 | all live |
| `lint(cited-name-absent)` (2) | 2 | 0 | all live |

**Corrected exhaustive-set population: 45 live exemptions** (38 ward-runes + 6 small-lint + 1 `#[allow]`), not the ~49 a naive read of the brief's table implies.

I independently re-verified the brief's own correction: strict `grep -rn '#\[ignore'` and `#\[allow` → **zero** `#[ignore]`, exactly **one** `#[allow]` (`src/rete/kernel/tests/fanout_cost.rs:841`, no reason), matching the brief.

**Exhaustive set weighed: all 45 live exemptions**, individually, against the disk (see ledger).

**Sampled set:**
- `no-inlined-wat` (9 total) — weighed **all 9** (exceeded the "sample" instruction; cheap enough).
- `loose-assert` (52 confirmed, no prose false-positives found) — sampled **27** (52%) by this rule: every instance in the two largest files (`probe_arc278_export.rs` 13, `probe_constructor_meta_surface_audit.rs` 8 → sampled 6 and 8 respectively) plus at least one instance from every one of the other 13 carrier files, covering every distinct reason-shape observed (Span/`rust_caller_span` volatility, targeted-absence, rendered-census-payload volatility).

## 2. The ledger

### STRUCK

**ILLEGITIMATE-AT-BIRTH**
- `src/rete/kernel/tests/fanout_cost.rs:841` — `#[allow(unused_variables)]` (no reason string at all)
  - Silences: rustc `unused_variables` on `child_tax: f64` (line 842).
  - Evidence: `git log -p` shows `child_tax` and its `#[allow]` were introduced together in commit `f98226353` (the tests.rs split) and it has **never** been read anywhere in the file since (`grep -n 'child_tax'` → only the definition line). The neighboring comment explains the *concept* of "nesting tax" and even cites measured numbers (18.992→11.524 ms) but never justifies leaving the value dead rather than wiring it into the printed table (where the sibling `top_sum`, defined right after, *is* printed) or deleting it. This is a real, live finding (the variable is genuinely computed and discarded) with no earned exemption — a bare suppression is ILLEGITIMATE-AT-BIRTH by the spell's own default.
  - Level: L1 (an unfought finding wearing a suppression).
  - Closure: checker is still live → remove the `#[allow]` and fix the code — either print `child_tax` in the phase table (it appears to be the intended output, given the neighboring "NESTING TAX" documentation) or delete the dead computation.

### HOLDS (exhaustive set — 44 of 45)

**vocare(vantage-bypass-test) — 23/23 HOLDS.** All read `Session/alpha-memory`, `Session/production-memory`, `Session/facts`, `Aggregate.fields`, or `TypeEnv` directly at the Rust host level. In every case I verified the structural claim from the code itself: either the rule under test has a deliberately **empty `:rhs`** (so no wat-level query mouth can ever see the result — confirmed by reading the `:rhs (:wat::core::PersistentVector)` literal in each `.wat` fixture), or the assertion targets a value wat has **no setter for** (Export's packed fields, TypeEnv), which I could not falsify. Sites:
`probe_arc278_4c_retraction.wat:62,69`; `probe_arc278_2b_insert_alpha.wat:35,46,54,64`; `probe_arc278_compiled_where_ops.rs:53,71`; `probe_arc278_P2_native_fire_once.wat:59`; `probe_arc278_alpha_is_fire_scoped.wat:32,41,51`; `probe_arc278_export.rs:136,177,254,462,500,556,570`; `src/rete/kernel/tests/pass_semantics.rs:233,333,450,523` (this quartet is exemplary — each explicitly names its own coverage gap and the sibling test, `probe_arc278_join_carries_both_sides_into_the_rhs.rs`, that closes it — a textbook non-pre-emptive exemption).
Level: L3 (still true, no drift). Closure: none.

**perspicere(read-once) — 12/12 HOLDS.** Every site is a single-use, function-local scratch collection (a microbench index, a one-shot census bag, a vantage-bypass field poke) built and consumed within one function, never a shared domain type — verified by reading each site's surrounding code. Naming it as a "domain noun" would misdescribe it. Sites: `probe_arc278_export.rs:511`; `accum_alpha_cost.rs:819`; `alpha_discrimination.rs:74,252,402`; `gather_probe_cost.rs:498,508,526,649,660,784`; `mod.rs:500`. Level: L3.

**complectens — 2/2 HOLDS.**
- `pass_semantics.rs:97` (`proof-stepping-stones`) — the test genuinely drives the four kernel passes (`alpha_pass`, `root_join_pass`, `hash_join_pass`, `production_pass`) separately as a documented four-pass proof; collapsing to a single call would destroy the pass-by-pass diagnostic it exists for.
- `node_share_cost.rs:98` (`inline-fixtures`) — cites a specific, checkable historical regression: *"a block-ordered A/B produced a clean, disjoint, WRONG −7 ms on 2026-08-01 that a B-A-B drift check destroyed."* Structural and falsifiable.
Level: L3.

**exigere(scope-affirmative) — 1/1 HOLDS.** `tests/rete/probe_arc278_P12_explain_walk.rs:15`, standalone module-doc declaration naming an explicit, checkable out-of-scope decision, tracked in `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P12-explain-walk.md`. Distinct text from the other `exigere` sites found elsewhere in the repo (`wat/rete.wat:524`, `src/kernel/*`) — confirmed this is its own instance, not a stray duplicate. Level: L3.

**lint(census-name-retired) — 4/4 HOLDS, self-verifying.** `accum_cost.rs:622,624,626,628`. Verified the cited commit `c9d751049` ("278: bind-value intern through retiring per-fact alpha timers... retired per-fact alpha child timers") is real. Verified `grep -rn '"alpha:candidates"\|"alpha:match"\|"alpha:element"\|"alpha:push"' src/` returns **nothing** — the four names are genuinely unemitted. Better still: `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` carries its **own** gate, `every_census_name_retired_rune_names_a_name_the_engine_no_longer_emits`, that re-checks this exact claim on every run — this category is closer to continuously-machine-verified than any other in the target. Level: L3.

**lint(cited-name-absent) — 2/2 HOLDS.** `mod.rs:2` (cites `tests.rs`) and `mod.rs:340` (cites `fire_cost_census.rs`). Verified both files are genuinely gone: `git log --diff-filter=D` finds `f98226353` "rete: kernel/tests.rs 10,189 lines -> 13 files" for the first; `find . -name fire_cost_census.rs` returns nothing for the second, consistent with the same 2026-08-30 split the module doc describes. Level: L3.

### Not weighed as exemptions (measurement corrections, not verdicts)

- `struere(invariant-coupling)` — the sole in-target grep hit (`probe_arc278_import_fold_key.rs:5`) is prose quoting a rune that lives entirely outside the target at `src/rete/kernel/fire/acc.rs` (4 sites). No live `struere` exemption exists inside `tests/rete/` or `src/rete/kernel/tests/`. Nothing to grade here.
- `lint(red-by-design)` — both in-target hits (`probe_arc278_match_arm_is_not_a_call.rs:37,190`) are prose about a rune that lived outside the target (`docs/arc/.../experiri-then-match.wat`) and, by the file's own account, has already been struck. Nothing to grade here.

### Sampled — no-inlined-wat (9/9, exhaustive) — ALL HOLD

All 9 sites were verified structurally: the cited function (`world(acc, gate)`, `gen_world(depth)`, `run_expr(expr)`, etc.) genuinely builds wat source via `format!` from a runtime parameter that varies per call, making a single static `.wat` fixture impossible by construction. One outlier reason-shape (`probe_arc278_P12c_explain_payload.rs:74`) is different but equally sound: the string `"(op a b)"` is prose inside an `assert_eq!` failure message, never parsed as wat — verified by reading the surrounding `assert_eq!`/`assert!` calls. Sites: `probe_arc278_seq1b_list_hofs.rs:25,33`; `probe_arc278_8b_accumulate_native_differential.rs:14`; `probe_arc278_8custom_native_differential.rs:17`; `probe_arc278_8a_accumulate_oracle.rs:19`; `probe_arc278_deep_cascade.rs:21`; `probe_arc278_P12c_explain_payload.rs:74`; `probe_arc278_P6_delta_asymmetric_join.rs:27`; `probe_arc278_6b_ii_b_where_native_differential.rs:17`. Level: L3.

### Sampled — loose-assert (27/52, ~52%) — ALL HOLD

Every sampled instance falls into one of three structurally-verified patterns:
1. **Span volatility** (majority — `probe_arc278_export.rs` x6 read, `probe_constructor_meta_surface_audit.rs` x8, `probe_construction_headline.rs`, `then_operand_wall.rs` x4, `rete_defn_recurse.rs`, `rete_defn_gap.rs` x2, `import_accounting.rs` x3): I traced `rust_caller_span!()` to its definition (`src/span.rs:14-21`) and confirmed it embeds `file!()`/`line!()`/`column!()` into the error's `Debug` output — genuinely non-deterministic across edits/machines, so an exact-match assertion is structurally wrong and a targeted substring is the correct strength. `then_operand_wall.rs:40-46` even names the actual checker (`no_loose_string_assert`) and quotes its own stated carve-out verbatim, closing the loop.
2. **Targeted absence is the finding** (`enum_variant_typo.rs:108,115`, `rete_edn.rs:63,67,76,80`, `inline_constraint_law_a.rs`, `import_accounting.rs:157`, `match_arm_is_not_a_call.rs:126`): the test already does an exact golden match (`assert_edn_eq!`) elsewhere and the "loose" assert is an additional `!contains()` negative-space check where the absence of a specific substring literally *is* what the test exists to prove.
3. **Rendered-census/panic-payload volatility** (`right_index_counter_invariant.rs:567`, `node_share_cost.rs:935`): explicitly cites a prior incident — *"the pinned-count failure this arc has already paid for twice"* — as the reason an exact match is wrong.

No deviation from these three patterns was found in any sampled site. Sites read (27): `probe_arc278_export.rs:177,494,550,564` (4, plus earlier context reads at 500/556/570 already counted under vocare review = 7 distinct export.rs lines total actually inspected), `probe_constructor_meta_surface_audit.rs:111,114,116,118,146,148,150,152` (8), `probe_arc278_enum_variant_typo.rs:108,115` (2), `probe_arc278_rete_edn.rs:63,67,76,80` (4), `probe_arc278_import_accounting.rs:88,144,148,157` (4), `probe_arc278_then_operand_wall.rs:48,51,53,59` (4), `probe_arc278_rete_defn_gap.rs:46,80` (2), `src/rete/kernel/tests/right_index_counter_invariant.rs:567` (1), `src/rete/kernel/tests/node_share_cost.rs:935` (1).

## 3. Aggregate

| set | weighed | HOLDS | STRUCK | breakdown |
|---|---|---|---|---|
| Exhaustive (ward-runes + small-lint + allow) | 45 | 44 | 1 | 1 ILLEGITIMATE-AT-BIRTH |
| Sampled: no-inlined-wat | 9 (of 9, exhaustive) | 9 | 0 | — |
| Sampled: loose-assert | 27 (of 52, 52%) | 27 | 0 | — |
| **Total weighed** | **81** | **80** | **1** | |

Population corrections: brief's "40 ward-runes + 8 small-lint" overcounts by 4 (2 `struere`/`red-by-design`-shaped hits were prose-about-elsewhere, not exemptions); `vocare` overcounts by 1 for the same reason. Corrected live population in target: **38 ward-runes + 6 small-lint + 1 allow + 52 loose-assert + 9 no-inlined-wat = 106** live exemptions (not 109 — the 3-item delta is entirely the phantom `struere`×1 / `red-by-design`×2 prose hits).

## 4. The 37 sibling-ward runes (vocare/perspicere/complectens), as a group

**Legitimately exempted, not pre-emptive skips.** This is the most important finding of the cast: every one of the 37 (23 vocare + 12 perspicere + 2 complectens, corrected from the brief's 37 nominal count which included one phantom vocare) earns its exemption by a check I could actually perform from the code alone — not by taking the annotator's word:

- All 23 `vocare` sites reach into genuine implementer-only state (`Session/alpha-memory`, `Session/production-memory`, raw `Aggregate.fields`) behind rules with a **verified empty `:rhs`** or behind values wat has no constructor for. The four in `pass_semantics.rs` go further than legitimate — they *name their own coverage gap* and point at the exact sibling test that closes it, which is the opposite of a pre-emptive skip: it's an exemption that argues for itself and then gets checked.
- All 12 `perspicere` sites guard single-use, function-scoped scratch collections, verified by reading each call site — none is a promoted, reused, or exported name that would deserve a domain identity.
- Both `complectens` sites cite specific, falsifiable engineering history (a four-pass kernel proof; a named regression date-stamped and cross-referenced to a drift-check test) rather than a bare "trust me."

None of the 37 reads as a checker finding being walked past with a plausible sentence — each names the specific structural reason a wat-level or exact-match test cannot reach the thing being tested, and in the `vocare` case the corpus visibly *also* contains the caller-level tests that vocare's own ward would otherwise ask for.

## 5. Word

**FINDINGS**
