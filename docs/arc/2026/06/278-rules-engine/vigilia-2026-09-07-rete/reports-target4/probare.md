# PROBARE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

### 1. What I swept

```
find tests/rete -name "*.rs" | wc -l        → 100
find tests/rete -name "*.wat" | wc -l       → 144
find tests/rete -name "*.edn" | wc -l       → 23
find tests/rete -name "*.wat.bad" | wc -l   → 19
find src/rete/kernel/tests -name "*.rs"     → 20
cat all of the above | wc -l                → 38058   (27135 .rs + 9036 .wat + 508 .wat.bad + 1379 .edn)
grep -rn '#\[test\]' … | grep -v commented  → 613
```
All three headline figures (306 files, 38,058 lines, 613 tests) **match exactly** — no deltas from the brief this time. The 613 figure required excluding 2 lines where `#[test]` appears inside a `//` comment (`probe_arc278_P4c_native_retraction.rs:18`, `probe_arc278_P4a_native_fire_rules.rs:16`), same trap class as prior art #1.

**Self-claim grep, widened.** Original narrow pattern re-run: **30**, confirmed exact match to the brief. Widened to add `%`, spelled-number `X of Y`, bare `N tests/probes/fixtures/…`, `always/never/only one/none of`, and ran over **all four extensions**, restricted to comment lines (`//`,`///`,`//!` for `.rs`; `;;` for the rest): **266 hits in `.rs`, 98 in `.wat`/`.wat.bad`/`.edn`** — 364 total. That is far too many to derive individually; I triaged by specificity, prioritizing exact fractions (`N of M`) and git-checkable history claims, which are the highest-value/lowest-ambiguity class per the brief's own steer.

**Sampling rule for `.wat` density:** systematic sample, every 5th file of 144 sorted alphabetically → 29 files (20.1%). I did not read the other 115 `.wat` bodies for density purposes. I did not open `wat-scripts/perf/grid/` (outside scope — see §5).

### 2. Density assay

**`.rs` (all 120 files, full corpus, not sampled):**
- Total 27135 / comments 6430 / blank 1714 / code (non-blank, non-comment) 18991
- Primary ratio (code-lines : comment-lines, the substance-vs-prose split appropriate for Rust test bodies where the substance lives in statements/assertions, not just declarations): **18991:6430 ≈ 2.95:1 → MIXED** (just under the 3:1 substance-rich line).
- Secondary/stricter metric (declaration-only lines — `fn/struct/impl/use/mod/enum/trait/const/static`): 1496:6430 ≈ 0.23:1, which would misfile as "wish." I discount this reading: it undercounts by design for Rust test files, where most substance is intra-body statements, not declarations — noted as a caveat, not used as the verdict.

**`.wat` (29-file / 20.1% systematic sample):**
- Total 1743 / comments (`;;`) 430 / blank 239 / forms (lines opening `(`) 831
- Ratio 831:430 ≈ **1.93:1 → MIXED**

**Verdict, stated plainly: MIXED for both languages.** This is a clean result in the sense the spell defines "mixed" — spec + commentary, purpose-appropriate for a probe/fixture corpus this heavily narrated. Nothing here crosses into "prose-heavy" or "wish." No rename/restructure recommendation. I found **zero `rune:probare(...)` markers** anywhere in either tree, so there is nothing to weigh a reason against.

### 3. Self-claim table (derived against disk)

| claim | location | derived truth | verdict |
|---|---|---|---|
| KIND N of 4 (`UnknownField`, `RhsMissingFields`, `RhsArityMismatch`, `RhsPositionalConstructionRetired`) | `probe_arc278_nested_wall_{unknown_field,missing_fields,arity,positional_retired}.wat:1` | Exactly 4 files, each claiming a distinct KIND 1–4 of 4, no 5th mismatch-kind file exists (`probe_arc278_nested_wall_ok.wat` is the non-error control, not a 5th kind) | **TRUE** |
| "all 8 HOFs must type-check" / runtime "for all 8 ops" | `probe_arc278_seq1b_list_hofs.rs:59, 13` | `list_hofs_typecheck_parametric` builds exactly 8 `defn` lines (l-foldl, l-fold-reverse, l-map, l-filter, l-rev, l-take, l-drop, l-concat); runtime section has exactly 8 dedicated `#[test]`s covering the same 8 ops | **TRUE** |
| "Fixed by routing … to `validate::render_form`" (4 sites) | `probe_arc278_then_operand_rendered_as_source.rs:12-13` | `git show 86091edf7` — commit exists, dated 2026-08-27, diff shows exactly 4 call sites (`eval_insert`×2, `compiled_rhs`×2 paths) converted to `crate::rete::validate::render_form(...)` | **TRUE** |
| "`g` WAS THE ONE MEAN AMONG SIX MINIMA … converted by `89e8c3ed0` and `g` was not" | `gather_probe_cost.rs:1042-1043` | `git blame` shows comment + fix land together in `119214aef5` (2026-08-31, one day AFTER `89e8c3ed0`, 2026-08-30). `git show 119214aef5` diff: `g` changed from `+= …; g /= runs` (a mean) to `g.min(...)`. `89e8c3ed0`'s own message confirms it converted "106 accumulators" but not this file's `g`. Narration is exact. | **TRUE** |
| 21 sampled commit-hash citations | scattered, see §4 | `git cat-file -e` + `git log -1` — every hash resolves to a real commit whose subject line topically matches the citing comment's claim | **TRUE** (21/21 that were actually hashes) |
| `2097268`, `545259536` "commit-like" hits | `probe_arc278_import_accounting.rs:11`, `probe_arc278_fixpoint_round_cap.rs:14` | Read in context: both are byte counts from an allocation measurement, not commit hashes — my hex-shaped grep pattern matched digit strings that happen to look hash-like | **not a claim** (grep false positive, correctly excluded, not reported as a finding) |
| "371 of 381 corpus rules" | `termination_verdict.rs:94-95` | The file's own `:a5p`/`:a5v`/`:a5d` fixture corpus is 3 tiny synthetic namespaces (1 rule each), not 381 rules. No "381" appears anywhere else in either target tree. The claim's population is external | **unverifiable-as-stated within scope** |
| "115 fixtures fire the branch pair; 39 reach obligation 1 and 34 reach obligation 2 … 528 skipped pairs … 34 of 115" | `where_tree_branch_differential.rs:50-54` | `walk_corpus()` (same file, line 428) reads its population from `grid_dir()` = `wat-scripts/perf/grid/where-*.wat`, which is **outside both target trees** | **unverifiable-as-stated within scope** |
| "a guard at slot 3 of 5 failed while the same guard at slot 4 of 5 worked" | `probe_arc278_where_is_positionally_free.rs:21`, companion `.wat:19-20` | The file's own 4 fixture rules (`r-one`, `r-two`, `r-four`, `r-trail`) have `:when` lists of length 4, 4, 6, and 4 respectively — none has exactly 5 `:when` elements, so a literal "slot 3 of 5"/"slot 4 of 5" count over *this file's* conditions doesn't fit any rule shown. Surrounding text ("THE TRIGGER IS NOT WHAT THE REPORT THOUGHT") reads as narrating the **original external bug report** (filed 2026-08-24, in a NOTE- doc outside scope), not this minimized reproduction | **unverifiable-as-stated** (likely narration of an out-of-scope investigation, not an assertion about this file's population — flagged per the trap warning rather than called false) |

### 4. History claims settled with git

- `86091edf7` and `119214aef5` above — both fully confirmed, diffs match their comments exactly.
- Spot-checked 23 additional commit-hash citations across `accum_cost.rs`, `right_index_counter_invariant.rs`, and fifteen `probe_arc278_*` files — all 21 real hashes exist and are dated/subject-matched to their citing context (e.g., `26c79470c` "strike: draw the nested-constructor wall — orphaned, not untaught" cited by `probe_arc278_nested_wall.rs`; `9d9a4e77` "alpha is 99.4% of it" cited by `probe_arc278_alpha_is_fire_scoped.rs`).
- Did **not** re-report the two already-confirmed history findings named in the brief (`task #94`, the "JVM tax paid ONCE" claim), per instruction.
- `task #50` (`node_share_cost.rs:97`) is a forward-looking design note, not an open/closed status claim — not git-checkable, correctly not pursued.

### 5. What I looked for and did not find

- No `rune:probare(...)` markers anywhere in scope.
- No file whose name/purpose is contradicted by its content.
- Widened self-claim grep (364 raw hits) was **not** exhaustively derived — I sampled toward the sharpest, most falsifiable subset (exact fractions + git-checkable history) per the brief's own steer, and did not individually chase the `always`/`never`/`only`/percentage hits.
- I did not open `wat-scripts/perf/grid/` or any file outside the two named trees, even though two headline numbers (`371 of 381`, `34/115`) depend on it — that's the finding in §3, not a gap I filled.

### 6. Verdict

**FINDINGS**
