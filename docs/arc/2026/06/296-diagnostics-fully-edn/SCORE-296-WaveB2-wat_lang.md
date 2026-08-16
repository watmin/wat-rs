# SCORE — 296 Wave B2 (`tests/wat_lang` + 1 `tests/types` prelude, 40 tests) — RECOVERED FROM TRANSCRIPT

> **This document is a RECOVERY, not a fresh adjudication.** The Wave B2 rider ran against
> `docs/arc/2026/06/296-diagnostics-fully-edn/BRIEF-296-WaveB2-wat_lang.md` earlier in this session,
> un-ignored all 40 tests, adjudicated every one, hit **STOP-2** (findings well over the brief's "~6"
> ceiling), and — by design, matching Wave B1's precedent — **captured nothing**. That adjudication was
> never written to disk (`SCORE-296-WaveB1-types.md` exists; there was no B2 sibling), so the 40
> verdicts existed only in this session's 120MB transcript, which had already survived one compaction.
> Recovered 2026-08-15 by mining the sub-rider's own transcript
> (`.../subagents/agent-a258413986ab6f49a.jsonl`, its final Summary message) — no code was touched, no
> tests were re-run, and no verdict was re-adjudicated. Where the recovered record is now known to be
> stale (the `not_eq_f64_cross_numeric_coerce` SUPERSEDED reclassification), that is flagged as a
> **comment on the row**, not a silent correction.

## THE RESULT (as reported by the original rider)

```
Summary [ 218.041s] 4606 tests run: 4566 passed (2 slow), 40 failed, 82 skipped
```

Predicted +40 run / −40 skipped: both exact. `passed` held at 4566 — nothing was captured, all 40
un-ignored tests are exactly the ones failing. Clippy: 0 warnings
(`cargo clippy --workspace --all-targets --release -- -D warnings`).

**31 staleness · 9 findings · 40 total.** STOP-2 fired (brief's "~6 findings" ceiling blown).

## THE ADJUDICATION TABLE

Format: `file :: test_fn | disposition | evidence/reason as the original rider stated it`.

The 30 plain-`STALENESS` rows below share one general reason, stated once by the rider rather than
per-row: the diff is the EDN Display-face transition only — `StartupError`'s `Display` impl switched
to emit EDN under Stone B (`src/freeze.rs:686`) — with every payload field, span, and error order
preserved. The rider noted the blanket ignore text ("rust-debug face") is imprecise about *which*
face changed for the `probe_arc241_stone*` / `probe_def_not_special` tests (they assert `format!("{}",
err)` / Display, not `{:?}` / Debug) but classified them the same staleness class regardless.

| file :: test_fn | disposition | evidence/reason (as stated by the original rider) |
|---|---|---|
| `tests/types/probe_arc293_holder_bound.rs :: core_record_rejected_by_holon_nature_bound` | **FINDING** | Message content correct (nature-mismatch on `:env::wants-holon` param#1), but span end **moved in the user's `.wat`**: old `end_col: 36` → new `Pos{:col 42}`. Verified char-by-char against `probe_arc293_holder_bound_reject.wat:13` — col 36 lands mid-token on `l` of `:slot` (nonsensical); col 42 is exactly one-past the closing paren of `(:env::CEnv :slot 1)`. Per THE LAW, a span that moved in the user's `.wat` is a FINDING regardless of which number "looks more correct." |
| `probe_arc234_stone4_hash_destructure.rs :: probe_5_unknown_field_errors` | STALENESS | face-only diff (see general note above) |
| `probe_arc241_stone11_define_hard_cut.rs :: contract_03_retirement_remedy_names_defn` | STALENESS | face-only diff |
| `probe_arc241_stone11_define_hard_cut.rs :: contract_04_retirement_kind_annotation_present` | STALENESS | face-only diff |
| `probe_arc241_stone11_define_hard_cut.rs :: contract_05_retirement_table_includes_define_entry` | STALENESS | face-only diff |
| `probe_arc241_stone12_defalias.rs :: contract_05_rejection_remedy_names_defalias` | STALENESS | face-only diff |
| `probe_arc241_stone13_define_dispatch_hard_cut.rs :: contract_02_rejection_remedy_names_defclause` | STALENESS | face-only diff |
| `probe_arc241_stone14_restricted_absorbed.rs :: contract_06_rejection_remedies_name_replacements` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_01_try_hard_cut_rejected` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_02_try_rejection_remedy_names_result_try` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_03_option_expect_lowercase_hard_cut_rejected` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_04_option_expect_lowercase_rejection_remedy_names_pascal` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_05_result_expect_lowercase_hard_cut_rejected` | STALENESS | face-only diff |
| `probe_arc241_stone15_zombie_purge.rs :: contract_06_result_expect_lowercase_rejection_remedy_names_pascal` | STALENESS | face-only diff |
| `probe_arc241_stone16_define_eval_residue.rs :: contract_01_define_rejection_carries_stone_241_16_marker` | STALENESS | face-only diff |
| `probe_arc241_stone16_define_eval_residue.rs :: contract_02_retirement_remedy_preserves_defn_replacement` | STALENESS | face-only diff |
| `probe_arc257_keys_destructure.rs :: probe_3_bare_symbol_brace_form_rejected` | STALENESS | face-only diff |
| `probe_def_not_special.rs :: probe_define_rejected_at_startup_check` | STALENESS | face-only diff |
| `wat_arc072_letstar_parametric.rs :: whitespace_inside_angle_brackets_raises_clean_lex_error` | STALENESS | rider's own qualifier: "internal `src/*.rs` span moved — kept per standing ruling" (i.e. an internal-diagnostic span move, not a user-`.wat` span move — treated as staleness per the standing ruling on that distinction, unlike the user-facing span moves below which were called FINDINGs) |
| `wat_arc136_do_form.rs :: do_empty_form_is_malformed` | STALENESS | face-only diff |
| `wat_arc136_do_form.rs :: do_recipient_mismatch_fires_type_mismatch` | STALENESS | face-only diff |
| `wat_arc143_define_alias.rs :: define_alias_retired_form_rejected_at_startup` | STALENESS | face-only diff |
| `wat_arc153_nil_rename.rs :: type_position_unit_post_retirement_is_unknown_fqdn` | STALENESS | face-only diff |
| `wat_arc153_nil_rename.rs :: value_position_nil_against_i64_recipient_fires_type_mismatch` | STALENESS | face-only diff |
| `wat_arc153_nil_rename.rs :: reverse_mixed_nil_body_with_retired_unit_sig_post_retirement` | STALENESS | face-only diff |
| `wat_arc153_nil_rename.rs :: bare_legacy_unit_name_walker_retired` | STALENESS | face-only diff |
| `wat_arc154_kill_let_star.rs :: let_star_post_retirement_silently_aliases_to_let` | STALENESS | face-only diff |
| `wat_arc154_kill_let_star.rs :: let_body_type_mismatch_surfaces` | STALENESS | face-only diff |
| `wat_arc154_kill_let_star.rs :: multiple_let_star_sites_post_retirement_silently_alias` | STALENESS | face-only diff |
| `wat_arc157_def.rs :: def_type_mismatch_via_registered_type` | **FINDING** | span moved in user `.wat`: old `col:71 end_col:74` → new `col:71 end_col:77`. Fixture line 3 argument is `:t::pi`; old span (71–73) = `:t:` (truncated token); new span (71–76) = `:t::pi` (full, correct token) |
| `wat_arc157_def.rs :: def_type_error_in_expr` | **FINDING** | span moved in user `.wat`: old `col:35 end_col:47` → new `col:38 end_col:50`. Fixture line 4 is `(:wat::core::def :t::bad (:t::helper "not-an-int"))`; old span (35–46) = `er "not-an-i` (mid-token garbage spanning "helper" into the string); new span (38–49) = `"not-an-int"` (the true argument-literal span) |
| `wat_arc157_def.rs :: def_redef_forbidden_strict_default` | **FINDING** | payload field `:name` changed *value*, not format — fixture `wat_arc157_def_redef_forbidden.wat.bad`: old `":a"` → new `":wat-arc157-def-redef-forbidden::a"`, post an unrelated prior `72a1ac3d` namespacing codemod; spans identical (3:2–17, prior 2:2–17) |
| `wat_arc157_def.rs :: def_redef_default_flag_off_strict_default` | **FINDING** | same fixture/cause as `def_redef_forbidden_strict_default` above (same `wat_arc157_def_redef_forbidden.wat.bad`) |
| `wat_arc157_def.rs :: def_redef_set_redef_true_type_change_fires` | **FINDING** | fixture `wat_arc157_def_redef_type_change.wat.bad`: old `":a"` → new `":wat-arc157-def-redef-type-change::a"`; spans identical (4:2–17, prior 3:2–17) |
| `wat_arc157_def.rs :: def_redef_set_redef_false_strict_default` | **FINDING** | fixture `wat_arc157_def_redef_false.wat.bad`: old `":a"` → new `":wat-arc157-def-redef-false::a"`; spans identical (4:2–17, prior 3:2–17) |
| `wat_arc168_let_flat_shape.rs :: odd_count_vector_errors` | STALENESS | face-only diff |
| `wat_arc168_let_flat_shape.rs :: multi_form_let_body_typecheck` | STALENESS | face-only diff |
| `wat_idempotent_redeclare.rs :: typealias_divergent_errors` | STALENESS | face-only diff |
| `wat_idempotent_redeclare.rs :: define_divergent_body_errors` | **FINDING** | span AND file changed. Old literal: `file: "wat/core.wat", line: 512, col: 9, end_col: 24` for both the error span and `original_def_span` (implying collision with a builtin in `wat/core.wat`). New actual: `file: "tests/wat_lang/wat_idempotent_redeclare_define_div.wat.bad"`, error at line 5 col 1–94, prior-loc at line 4 col 1–94 — both inside the fixture, which defines `:my::add-one` twice with different bodies. Fixture text supports the new behavior as correct for what's written, but this is a substantive divergence from the old expectation |
| `wat_not_eq.rs :: not_eq_f64_cross_numeric_coerce` | **FINDING** (see comment) | Original rider's verdict, verbatim: *"the serious class: an error disappeared entirely."* Test asserts `result.is_err()` on `(:wat::core::not= 3 3.0)` (arc-237 Stone 237.8a: "cross-numeric coercion for equality DELETED," per the test's own comment) and now gets `Ok`. Rider flagged it to the orchestrator with priority as the same Class-D "check that no longer fires" species that opened a security stone in Wave B1. **COMMENT (post-hoc, from `docs/arc/2026/06/255-builtin-registry/SEAM.md`, not part of the original recovery):** this has since been reclassified **SUPERSEDED**, not FINDING — arc **300 Stone C5** deliberately reversed 237.8a's cross-numeric-coercion deletion to match `eval`/clj semantics; the check didn't silently break, the design under it changed. SEAM.md: *"I called this 'the serious class — a check that no longer fires,' the same label a real security hole got hours earlier. From inside a test the two are identical; only the record discriminates."* This row is left as **FINDING** here because that is what the original rider concluded — see the counts/deltas section below for how this affects the 31/9 comparison. |

## COUNTS SUMMARY

- **Recovered: 40 / 40.** Zero UNRECOVERED (see below).
- Original rider's split: **31 STALENESS / 9 FINDING** (8 in `tests/wat_lang` + 1 prelude in `tests/types`).
- **Matches the seam's 31/9 exactly**, as reported. No numeric disagreement to flag on the raw count.
- **But the composition of the "9" has since moved, per SEAM.md**: one of the 9 (`not_eq_f64_cross_numeric_coerce`) is now recorded as **SUPERSEDED**, a third disposition distinct from FINDING that the original Wave B2 brief didn't carry as a category. SEAM.md's own arithmetic treats "9 findings" as shorthand for "9 not-captured," with the finer split (8 FINDING + 1 SUPERSEDED) applied only in the later record. So: **31 STALENESS + 8 FINDING + 1 SUPERSEDED = 40**, which is consistent with both this recovery and the seam, once the SUPERSEDED column is applied.
- SEAM.md also flags, as an open re-brief item (not yet ruled, **not applied to this recovery's table**): of `wat_arc157_def.rs`'s 6 findings, 4 (the `def_redef_*` namespacing-desync ones) are called out as *"a named cause, likely plain staleness"* pending a fresh look — i.e. SEAM.md suspects some of the 6 may ultimately reclassify to STALENESS once someone re-examines whether the namespacing-codemod desync is cosmetic. This recovery does **not** act on that suspicion — it is recorded here as a pointer for whoever re-briefs, per the task's "do not re-adjudicate" rule.

## THE FINDINGS — VERBATIM

**1. Prelude — `tests/types/probe_arc293_holder_bound.rs :: core_record_rejected_by_holon_nature_bound`** (span moved in user `.wat`)
> New EDN correctly cites the nature-mismatch (`:env::wants-holon` param#1 expects `:env::Holon`, got `:env::CEnv`) — the message content is right, as the brief predicted. But the span end moved in the user's `.wat`: old literal `end_col: 36`, new EDN `Pos{:col 42}`. Verified against `probe_arc293_holder_bound_reject.wat` line 13 char-by-char: col 36 lands mid-token on `l` of `:slot` (nonsensical); col 42 is exactly one-past the closing paren of the argument expression `(:env::CEnv :slot 1)`. Per THE LAW ("a span that moved in the user's `.wat` is a FINDING"), this must be reported, not blessed — regardless of which number looks "more correct." Not captured; still un-ignored and red.

**2–5. `tests/wat_lang/wat_arc157_def.rs` — the four `def_redef_*` tests** (payload field `:name` changed value, NOT a format change)
> All four fixtures now use fully-namespaced names post the (unrelated, prior) `72a1ac3d` codemod that the file's own doc comment credits with namespacing "the six `wat_arc157_def_*` fixtures." The ignored tests' inline literals were never updated to match:
> - `def_redef_forbidden_strict_default` / `def_redef_default_flag_off_strict_default` (same fixture `wat_arc157_def_redef_forbidden.wat.bad`): old `name: ":a"` → new `:name ":wat-arc157-def-redef-forbidden::a"`. Spans identical (3:2–17, prior 2:2–17).
> - `def_redef_set_redef_false_strict_default` (`wat_arc157_def_redef_false.wat.bad`): old `":a"` → new `":wat-arc157-def-redef-false::a"`. Spans identical (4:2–17, prior 3:2–17).
> - `def_redef_set_redef_true_type_change_fires` (`wat_arc157_def_redef_type_change.wat.bad`): old `":a"` → new `":wat-arc157-def-redef-type-change::a"`. Spans identical (4:2–17, prior 3:2–17).

**6. `wat_arc157_def.rs :: def_type_error_in_expr`** (span moved in user `.wat`)
> Old `col:35 end_col:47`; new EDN `col:38 end col:50`. Fixture line 4 is `(:wat::core::def :t::bad (:t::helper "not-an-int"))`. Verified by column: old span (35–46) = `er "not-an-i` (mid-token garbage spanning "helper" into the string); new span (38–49) = `"not-an-int"` exactly — the true argument-literal span.

**7. `wat_arc157_def.rs :: def_type_mismatch_via_registered_type`** (span moved in user `.wat`)
> Old `col:71 end_col:74`; new `col:71 end_col:77`. Fixture line 3 argument is `:t::pi`; old span (71–73) = `:t:` (truncated token); new span (71–76) = `:t::pi` (the full, correct token).

**8. `wat_idempotent_redeclare.rs :: define_divergent_body_errors`** (span AND file changed — the largest structural finding)
> Old literal: `file: "wat/core.wat", line: 512, col: 9, end_col: 24` for *both* the error span and `original_def_span` — implying `:my::add-one` collided with a builtin registered in `wat/core.wat`. New actual: `file: "tests/wat_lang/wat_idempotent_redeclare_define_div.wat.bad"`, error at line 5 col 1–94, prior-loc at line 4 col 1–94 — both inside the fixture itself, which defines `:my::add-one` twice (lines 4–5) with different bodies. The fixture text supports the *new* behavior as correct for what's written, but this is a substantive divergence from the old expectation, not a format change — reported as required, not captured.

**9. `wat_not_eq.rs :: not_eq_f64_cross_numeric_coerce`** — "the serious class: an error disappeared entirely"
> Test asserts `result.is_err()` on `(:wat::core::not= 3 3.0)` (arc-237 Stone 237.8a: "cross-numeric coercion for equality DELETED" per the test's own comment) and now gets `Ok`. The type-check gate this test exists to prove no longer fires — startup succeeds where it should reject a same-type-only relational intrinsic given cross-numeric operands. This is the one finding here in the same category batch1's Class D ("a check that no longer fires at all") that opened a security stone — worth flagging to the orchestrator with priority.
>
> **Superseded per later record (SEAM.md, not part of the original rider's report):** arc 300 Stone C5 deliberately reversed 237.8a. The disappearance is a design change, not a broken check.

## STOPs FIRED

**STOP-2** — findings (9) exceed the brief's "~6 findings in this batch" ceiling. Per doctrine and Wave B1's precedent (`SCORE-296-WaveB1-types.md`: *"PARKED... nothing was captured"*), work halted before any conversion or capture, even for the 31 clean-staleness tests. All 40 tests remain un-ignored and red on the floor — the deliberate, honest, uncaptured state, not an unintended red.

## HONEST DELTAS AGAINST THE BRIEF (as the original rider reported them)

- File/test counts matched the disk exactly (no drift, unlike prior briefs in this campaign): 39 across 18 files in `tests/wat_lang` + 1 in `tests/types`, summing to 40 (rider's arithmetic: `6+6+4+3+3+2+2+2+2+1×9=39` for wat_lang).
- The brief's confident framing of the prelude ("now it produces the correct nature-bound rejection... closes tests/types at 34/34... ordinary T2") undersold it — it's a finding (span moved), not a clean conversion.
- Several `probe_arc241_stone*` / `probe_def_not_special` tests assert `format!("{}", err)` (Display), not `{:?}` (Debug) — the blanket ignore reason says "rust-debug face" but `StartupError`'s `Display` impl was ALSO switched to emit EDN under Stone B (`src/freeze.rs:686`), so these are the same staleness class despite the reason text being imprecise about which face.
- `wat_arc157_def.rs` turned out to be the single worst-affected file: all 6 of its ignored tests are findings (4 from an unrelated prior namespacing codemod desyncing fixture content vs golden, 2 from genuine span-move regressions) — this file alone exceeds the batch's STOP-2 threshold.

## HOLLOW-FIXTURE CHECK / NEGATIVE CONTROLS (as reported)

None found. Every test in this batch produced a real, substantive checker/runtime result. No test in this batch is a standalone negative control in the DUNGEON-CRAWL Phase-3 sense; the un-ignore itself is the only banking action available and is already done for all 40.

## BLAST RADIUS / STATE LEFT BEHIND (as reported, and matching the current uncommitted checkout)

Only `#[ignore]` attribute deletions — 40 lines across 19 files (18 in `tests/wat_lang`, 1 in `tests/types`), verified via `git diff --stat`. No `.edn` goldens written, no `src/` touched, no `.wat` corpus touched, no `git commit`/`stash`/`checkout` run.

## UNRECOVERED

None. All 40 tests named in the ground-truth diff (`git diff HEAD --name-only -- tests/`, 19 files) have a
recovered disposition above, cross-checked one-for-one against the file/fn list.
