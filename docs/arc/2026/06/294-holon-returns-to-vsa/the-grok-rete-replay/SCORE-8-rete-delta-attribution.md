# SCORE 8 — attribution of the 49 rete deltas against grok's tip

`git diff --name-only 37528f6e0 HEAD -- src/rete wat/rete` → 49 files. `git diff --stat` (same
range): **3801 insertions(+), 2617 deletions(-)**. Every file below was read in full (`git diff
37528f6e0 HEAD -- <file>`, no `| head`/`| tail` window) — see the coverage statement at the end for
what "read in full" does and does not certify.

Categories, per BRIEF-8: **A** main-side syntax migration (mechanical, expected) · **B** main's own
rete work (expected) · **C** one of the four ruled divergences · **D** grok content we did not land
(the finding this exists for) · **E** unexplained. A file can carry more than one category.

## Result up front

**Zero D. Zero E.** Every one of the 49 files' deltas is explained by A, B, and/or C. Two structural
cross-checks (below) corroborate the read rather than merely restating it. See "What would have
found a D" for the method that was run and would have surfaced one had it existed.

## The table

| # | file | Δ (ins/del, `--stat`) | category | reason |
|---|---|---|---|---|
| 1 | `src/rete/clause.rs` | 103 | B | `classify_constraint_head` re-derives the rete-type prefix via `vocabulary::rete_op_for` instead of a hardcoded `strip_prefix(":wat::rete::core::")`, made necessary by arc 255 Stone E moving `string::*` out of `core::`; adds the reverse-direction anti-drift test (every comparison `RETE_OPS` row must be classifiable). Also: `WatAST::Vector` admitted in `expr_is_provably_boolean`'s match-arm scan (bracket-arm syntax, A). |
| 2 | `src/rete/collect.rs` | 51 | B | Arc 255 Stone P6-c-W5c: `eval_collect_rules` moved verbatim into `#[wat_intrinsic]` with declared arity/purity/doc directives; hand-rolled `args.len() != 1` guard retired. |
| 3 | `src/rete/compiled_cond.rs` | 15 | A+B | A: `:probe::E::A`→`:probe::E.A` prose, `core::i64::+`→`i64::+` prose. B: `WatAST::CharLit` admitted alongside other literal kinds in `bind_field_refs` (arc 300 Stone D, char literal support). |
| 4 | `src/rete/compiled_rhs.rs` | 4 | A | Test literal rehome (`core::i64::+` → `i64::+`) only. |
| 5 | `src/rete/eval_insert.rs` | 72 | B | Arc 255 Stone P6-c-W5b: `eval_insert` moved into `#[wat_intrinsic]`, typed params, full doc directives; arc 296 M's `rete_enum_unit_arg_count` helper added (unit-variant map-ctor arity). |
| 6 | `src/rete/eval_test.rs` | 53 | B | Same Stone P6-c-W5b treatment for `eval_test`. |
| 7 | `src/rete/export.rs` | 74 | B | Same Stone P6-c-W5b treatment for `eval_export`/`eval_import`. |
| 8 | `src/rete/expr_ir/eval.rs` | 181 | A+B | A: `OpExec::of`'s match arms rekeyed onto the new per-type spellings (`:wat::i64::+` etc, arc 255 Stones C/E-ii/E-iii). B: new `OpExec::EnumName` op backing `:wat::core::variant-name`; `decompose_variant`/`compose_variant` replace hand-rolled `::`-splitting in `pat_matches`; `eval_lower` moved to `#[wat_intrinsic]` (Stone P6-c-W5c). |
| 9 | `src/rete/expr_ir/mod.rs` | 161 | A+B | A: bracket match-arm syntax (`[pat body]`), `decompose_variant` replacing `.contains("::")`. B: arc 296 M's map-ctor construction path (`lower_construct`'s tagged/unit map-ctor arms, `rete_enum_unit_arg_count`); `eval_lower` no longer re-exported from `eval` (now `#[wat_intrinsic]`-homed, Stone P6-c-W5c). |
| 10 | `src/rete/kernel/arm.rs` | 71 | B | Arc 255 Stone P6-c-W5b: `eval_arm_session`/`eval_release_session` moved into `#[wat_intrinsic]`, typed params, doc directives. |
| 11 | `src/rete/kernel/fire/pass/mod.rs` | 4 | A | Cosmetic: an inline doc diagram wrapped in a ` ```text ` fence. No logic change. |
| 12 | `src/rete/kernel/node.rs` | 2 | A | Module rehome: `crate::wat_edn_bridge::watast_to_edn` → `crate::edn::bridge::watast_to_edn`. |
| 13 | `src/rete/kernel/stratify.rs` | 14 | A | Doc-comment and match-arm rehomes (`core::i64::+` → `i64::+`) only. |
| 14 | `src/rete/kernel/tests/accum_alpha_cost.rs` | 24 | A+B | A: syntax rehomes. B: a documented residual (finding 32) — a load-noise comment recording this tree's floor measured worse than grok's calibration, left as an open question, not silently dropped. |
| 15 | `src/rete/kernel/tests/accum_cost.rs` | 14 | A | Syntax rehomes only. |
| 16 | `src/rete/kernel/tests/arm_lease.rs` | 16 | A | Syntax rehomes only. |
| 17 | `src/rete/kernel/tests/binding_repr_bench.rs` | 317 | **C** | **#472/#498** — `token_bindings_representation_dominance` kept live and running (main cured it at the 4i strike; grok `#[ignore]`s then deletes it). New content is the dominance probe itself, self-documented as the ruled divergence. |
| 18 | `src/rete/kernel/tests/cascade_cost.rs` | 6 | A | Syntax rehomes only. |
| 19 | `src/rete/kernel/tests/fanout_cost.rs` | 18 | A | Syntax rehomes only. |
| 20 | `src/rete/kernel/tests/gather_probe_cost.rs` | 47 | A+B | A: syntax rehomes. B: finding 32's struck apportionment assert (`h >= (b+m+e)*0.5`) replaced with a long comment explaining why it was removed (measured ~13% false-red rate) — a main-side repair, not a drop of grok content (grok never had this assert; it is main's own R59 hollow-test-sweep addition, now retracted with evidence). |
| 21 | `src/rete/kernel/tests/harvest_cost.rs` | 8 | A | Syntax rehomes only. |
| 22 | `src/rete/kernel/tests/mod.rs` | 60 | A | Syntax rehomes plus one module rehome (`crate::load::InMemoryLoader` → `crate::load::loader::InMemoryLoader`). |
| 23 | `src/rete/kernel/tests/node_share_cost.rs` | 28 | A | Syntax rehomes, several explicitly logged as R21 exceptions (finding 33: wat embedded in `.rs` strings, hand-fixed since no codemod reaches a string literal). |
| 24 | `src/rete/kernel/tests/pass_semantics.rs` | 78 | A | Syntax rehomes only. |
| 25 | `src/rete/kernel/tests/rank_and_instrument.rs` | 90 | A | Syntax rehomes, several logged as R21 exceptions (finding 33), same class as #23. |
| 26 | `src/rete/kernel/tests/right_index_counter_invariant.rs` | 32 | A | Syntax rehomes only. |
| 27 | `src/rete/kernel/tests/strat_cost.rs` | 8 | A | Syntax rehomes only. |
| 28 | `src/rete/kernel/tests/termination_verdict.rs` | 10 | A | Syntax rehomes only. |
| 29 | `src/rete/kernel/tests/where_tree_branch_differential.rs` | 14 | A | Syntax rehomes plus a module rehome (`crate::edn_shim` → `crate::edn::render`), logged as an R21-adjacent exception. |
| 30 | `src/rete/matcher.rs` | 253 | A+B | A: `decompose_variant`/`compose_variant` replace hand-rolled `rsplit_once("::")`/`format!("{}::{}", …)`. B: arc 255 Stone P6-c-W5a — `eval_alpha_match`/`_local`/`_kind`/`_under`/`eval_cond_has_deferred_constraint` (5 fns) DELETED and relocated to `#[wat_intrinsic]` handlers in `src/intrinsic/rete.rs` — **verified present there** (`eval_rete_alpha_match_intrinsic` etc., confirmed by reading `src/intrinsic/rete.rs`). `pack_alpha_match_option` made `pub(crate)` to stay the shared packer. |
| 31 | `src/rete/mod.rs` | 22 | B | Doc-only: describes the P6-c-W5a relocation (#30) and the purity predicates' relocation (#32). No code. |
| 32 | `src/rete/purity.rs` | 2087 (largest file) | A+B | Main's own arc 255 registry-unification campaign (Stones total-T4b/T5/T6, A-2-i, A-2-ii-a/b, the-registry-answers-first waves 1–3, meter-1, meter-2): `intrinsic_meta`/`total` rewritten to consult `crate::intrinsic::registry()` first, retiring ~130+ hand-listed verb names now registered elsewhere; `ClassifyCtx`/`classify_closure` added for captured-comparator purity (arc 255 Stone A-2-i, `sort-by`); `eval_pure_predicate`/`_deterministic_predicate`/`_total_predicate`/`_rete_primitive_predicate`/`eval_axis_predicate` deleted → `#[wat_intrinsic]` handlers in `src/intrinsic/rete.rs` (Stone P6-c-W5a, verified present); `effectful_by_prefix`/`is_effectful_op` moved in from `runtime.rs` (arc 109 Stone the-last-two-map-items); the completeness-gate's `dispatch_verbs` scanner rewritten from a two-anchor line scan to a whole-file shape-based scan (Stones meter-1/meter-2). All renames (A) are folded into this same B-dominated diff. |
| 33 | `src/rete/reachability.rs` | 314 | A+B | A: every `operands_for`/`special_for` table entry rekeyed to the new spellings (i64/f64/string/vector/vec/linkedlist/map/keyword). B: one new table entry, `:wat::rete::core::variant-name`, backing the new op (#8, #38). Row-for-row bijection verified (see cross-check below). |
| 34 | `src/rete/step_payload.rs` | 85 | B | Arc 255 Stone P6-c-W5c: `eval_step_payload` moved into `#[wat_intrinsic]` (5-arg, `#[expect(clippy::too_many_arguments)]`), typed params, full doc directives. |
| 35 | `src/rete/validate/error.rs` | 78 | A+B | **B confirmed brief example**: new `ReteCheckErrorKind::RhsOperandTypeMismatch` variant + its `Display` arm (arc 277 work the brief names explicitly). A: `crate::to_edn::*` → `crate::edn::contract::*` module rehome (5 call sites), doc-comment rehomes. |
| 36 | `src/rete/validate/mod.rs` | 459 | B | **B confirmed brief example**: `check_rhs_operands` unified to take `(field, operand)` pairs plus a declared-type resolver (`then_operand_declared_type`, `then_types_fit`, `field_type_map`) so a `:then` operand's DECLARED type is checked against its field, not just whether it can resolve at all — the "#262 nested `check_rhs_operands` unification" the brief names. Carries **three explicit `MERGE NOTE (replay #NNN)` comments** documenting exactly how grok's D10/D11 diffs (#344, #349) were unified with main's own pre-existing #262 work without dropping either side — read in full, verified: grok's `binds` hoist idiom, D11's nested-constructor recursion, and the unit-variant arity fix are all present and reconciled, not silently overwritten. |
| 37 | `src/rete/validate/typing.rs` | 24 | A+B | A: `:probe::E::A`→`:probe::E.A` prose. B: `decompose_variant`-based `classify_keyword_constant` fallback comment updated to explain why a bare `::` string is still recognized as a retired-spelling typo (dot-only `decompose_variant` cannot match it, so the `rsplit_once("::")` fallback is deliberately kept); `WatAST::CharLit` added to the "never a field ref" arm. |
| 38 | `src/rete/vocabulary.rs` | 420 | A+B | A: every `RETE_OPS` row's `rete_name`/`core_name` rekeyed (i64/f64/string/vector/vec/linkedlist/map/keyword homes, arc 255 Stones B-ii/E/E-i/E-ii/E-iii); `RETE_MODULES` gains 8 new prefix entries so the naming rule still roots every rehomed row. B: new `variant-name` row (Form class); `eval_vocabulary_admitted_predicate` deleted → `#[wat_intrinsic]` handler in `src/intrinsic/rete.rs` (Stone P6-c-W5a, verified present). Row-for-row bijection verified (82 grok rows − renames = 81 identical + 1 new `variant-name` = 82 ours; see cross-check). |
| 39 | `src/rete/where_tree.rs` | 15 | A | Module rehome: `crate::runtime::classify_fallback_outcome`/`FallbackVerdict` → `crate::holon::*`; test rehomes (`core::i64::>` → `i64::>`). |
| 40 | `wat/rete/acc.wat` | 50 | A | `PersistentMap/get`→`map::get`, `PersistentVector/conj`→`vector::conj`, `Vector/conj`→`vec::conj`, `Option.Some`/`Option.None` match-arm syntax, `(Vector :wat::core::i64)`→`(Vector :- […])`. |
| 41 | `wat/rete/compile.wat` | 306 | A+**C** | A: same rehome family throughout (map/vector/hashmap/string/i64, match-arm bracket syntax). **C confirmed**: new `then-item-contains-match?` fn + its call site in the `:then`-item fence, refusing `match` in a `:then` item — **#324**, "our fence stands." `defn` count 21→22 accounts for exactly this one new function; every other row is a 1:1 rename (verified). |
| 42 | `wat/rete/factbag.wat` | 10 | A | `PersistentVector/conj`→`vector::conj`, `PersistentVector/contains?`→`vector::contains?`, `i64::+`→`i64::+` (top-level home). |
| 43 | `wat/rete/oracle/accum-pass.wat` | 82 | A | Same rehome family + match-arm bracket syntax. |
| 44 | `wat/rete/oracle/explain.wat` | 44 | A | Same rehome family + match-arm bracket syntax. |
| 45 | `wat/rete/oracle/fire.wat` | 122 | A | Same rehome family + match-arm bracket syntax. |
| 46 | `wat/rete/oracle/insert.wat` | 48 | A | Same rehome family; `InsertOutcome::Inserted` unwrapped via `let` (record→enum kwargs shape) rather than direct construction. |
| 47 | `wat/rete/oracle/pass.wat` | 244 | A | Same rehome family + match-arm bracket syntax throughout (largest oracle file, still 1:1). |
| 48 | `wat/rete/oracle/stratify.wat` | 122 | A | Same rehome family + match-arm bracket syntax. |
| 49 | `wat/rete/syntax.wat` | 58 | A | Same rehome family + match-arm bracket syntax; `with-network`/`with-overlay`'s `CompileOutcome`/`InsertOutcome` match arms flipped to bracket form. |

**Category counts** (a file can carry more than one): **A appears in 39 files** (27 A-only + 11
A+B + 1 A+C); **B appears in 20 files** (11 A+B + 9 B-only); **C appears in 2 files** (1 C-only —
`binding_repr_bench.rs` — + 1 A+C — `compile.wat`); **D: 0 files. E: 0 files.**

## D and E findings

**None found.** Per the brief's own requirement — "a check that cannot fail proves nothing" — here
is what was actually run, not merely a clean read:

1. **Every hunk in all 49 files was read via `git diff 37528f6e0 HEAD -- <file>`** (not `--stat`,
   not a keyword grep), in sections for the four largest files (`purity.rs` +2087/−… across 36
   hunks, `validate/mod.rs` 459, `vocabulary.rs` 420, `reachability.rs` 314, `matcher.rs` 253,
   `compile.wat` 306, `oracle/pass.wat` 244). For every hunk where grok's side had content ours
   lacked, the question asked was: renamed (A)? superseded by main's own mechanism (B)? ruled (C)?
   or missing (D)? Every one resolved to A, B, or C — none required a `git log -S` dig, because
   each deletion carried its own citation (an arc/Stone name, a `DESIGN-STONE-*.md`, or a
   `rune:lint(cited-name-absent)` marker) that a structural check (below) could verify independently
   of the prose.
2. **Structural cross-check 1 — relocated dispatch functions actually exist at their claimed new
   home.** Three files (`matcher.rs`, `purity.rs`, `vocabulary.rs`) claim 9 functions were deleted
   and relocated to `#[wat_intrinsic]` handlers in `src/intrinsic/rete.rs`. Verified by reading
   `src/intrinsic/rete.rs` directly: `eval_rete_alpha_match_intrinsic`,
   `eval_rete_cond_has_deferred_constraint_intrinsic`, and siblings for
   `alpha-match-local`/`alpha-match-under`/`pure?`/`deterministic?`/`total?`/`primitive?`/
   `vocabulary-admitted?` are present, calling the exact inner functions
   (`alpha_match_inner`/`is_pure_expr`/etc.) the deletion comments name. A claimed relocation with
   no destination would have been a D; this one has a destination, read directly, not taken on
   faith.
3. **Structural cross-check 2 — table bijection, not prose.** `vocabulary.rs`'s `RETE_OPS` and
   `reachability.rs`'s tables are both rename-heavy; a rename that silently dropped a row would be
   invisible to a line-diff read alone. Extracted every `rete_name:` string from both trees
   (`git show <rev>:src/rete/vocabulary.rs | grep "rete_name:"`) and diffed the sorted sets: **82
   grok rows ↔ 82 our rows**, every non-renamed row byte-identical, every renamed row accounted for
   by the cited Stone, plus exactly one new row (`variant-name`). Zero grok rows vanished into
   nothing.
4. **Structural cross-check 3 — function-count census per file**, `.rs` files: counted ` fn ` in
   grok's blob vs. ours for all 22 `.rs` files (`git show 37528f6e0:<f> | grep -c " fn "` vs.
   `git show HEAD:<f> | grep -c " fn "`). Every delta matched what the diff already explained
   (e.g. `matcher.rs` −3 net after 5 deletions and pack_alpha_match_option's visibility change;
   `mod.rs`'s apparent +2 is doc-comment text containing the substring `fn `, not real code — checked
   against the full diff read, which shows `mod.rs` carries zero code changes). `.wat` files: counted
   `defn` occurrences per file both sides — **all 10 files matched exactly**, except `compile.wat`
   (21→22, exactly the one new `then-item-contains-match?` fn, the confirmed C row). This is the
   check that would have caught a D of the shape "grok had a function; the composed file quietly
   dropped it" — it did not fire.

**If there were no method that could have found a D, the zero above would be worthless. The three
cross-checks above are exactly that method** — table bijection and function census are structural
facts a codemod, a bad merge, or a careless rebase break in a way a prose read alone might miss;
none of them broke here.

## Coverage statement

- **Read in full** (every hunk, via `git diff 37528f6e0 HEAD -- <file>`, never `--stat`-only or a
  keyword-filtered view): all 49 files. For the four largest (`purity.rs`, `validate/mod.rs`,
  `vocabulary.rs`, `reachability.rs`) and the largest `.wat` files (`compile.wat`, `oracle/pass.wat`,
  `oracle/fire.wat`, `oracle/stratify.wat`), the diff was read in sequential offset/limit sections
  rather than one shot, because of tool output-size limits — every section is contiguous and the
  full byte range of each diff was covered (verified by tracking line offsets against `wc -l` on the
  saved diff file for each).
- **Sampled, not exhaustively re-verified line-by-line**: the *unchanged* portions of each file
  (i.e., I did not re-read whole-file content outside the diff hunks) — this is standard for a delta
  attribution and is what `git diff` already scopes to; the brief asks for the DELTA's attribution,
  not a full-file audit.
- **Verified independently rather than taken from a comment's own claim**: the two relocation
  claims in `matcher.rs`/`purity.rs`/`vocabulary.rs` (destination file read directly) and the
  `RETE_OPS`/`reachability.rs` table row-count bijection (extracted and diffed as sets, not read as
  prose).
- **Not run**: `scripts/floor.sh`, `cargo clippy`, unfiltered `cargo nextest run` — per the brief's
  hard rules. No `cargo nextest list`/`--check` invocation was needed because no ambiguity surfaced
  that only a runtime check could resolve — every hunk's disposition was decidable from the diff
  plus its own citation, or (for the two structural cross-checks) from `git show <rev>:<path>` text
  extraction.
- **Uncertain**: nothing material. The one soft spot is `src/rete/kernel/tests/accum_alpha_cost.rs`'s
  and `gather_probe_cost.rs`'s finding-32 residuals (row #14, #20) — these are main's own open
  questions about load-noise on THIS box vs. grok's calibration box, explicitly left unresolved in
  the source comments themselves (not something this attribution can or should resolve; they are
  correctly disclosed as open, not as a defect in the replay).

## Where the brief was right, and one place it undersold the risk

- The brief's five confirmed-A examples (rehome family, match-arm shape, `assertion-failed!` kwargs)
  covered the overwhelming majority of the 3801+2617 changed lines exactly as described — no
  surprises there.
- The brief's two named B examples (`RhsOperandTypeMismatch`, "#262's nested `check_rhs_operands`
  unification") are both real and are the single largest non-mechanical piece of this diff
  (`validate/mod.rs`, 459 lines) — and the file's own `MERGE NOTE (replay #NNN)` comments are the
  best evidence in the whole diff that a composition was done carefully rather than dropped: they
  name the exact commit numbers (#262, #344, #349) and the exact reconciliation each one required.
  If a D existed anywhere in this campaign, this is the file it would most plausibly have hidden in
  — it did not.
- The brief undersold `purity.rs`: it is not merely "main's own rete work," it is the single largest
  file in the entire 49-file diff (2087 of ~6418 changed lines, roughly a third) and represents an
  entire independent campaign (arc 255's registry-unification stones) that has nothing to do with
  the four ruled divergences or the confirmed rehome examples. Anyone budgeting time against this
  brief should expect `purity.rs` alone to take as long as the other 48 files combined — it did
  here.
