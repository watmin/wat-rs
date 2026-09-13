# PILOT-LOG — grok-rete #1–#10 onto `replay/grok-rete`

Branch: `replay/grok-rete`. Source: `origin/grok-rete` `37528f6e0` (git show only). Base: `de827fb4c`.
Tooling: `419350022`. **Not pushed.** Main untouched.

Wall times for #6–#10 are stopwatch (UTC). Times for #1–#5 are reconstructed from commit timestamps plus the convert/floor logs from that session; convert of one new `.wat` was measured at 29 s.

## Steps

| N | C | kind | wall | files | conflicts / resolutions | tests (named PASS) | tricks |
|---|---|---|---|---|---|---|---|
| tooling | — | scripts | ~same commit as #1 | `scripts/replay/chain-order.sh`, `chain-order.overrides`, `convert.sh` | n/a | P1: `--no-overrides` == `bootstrap/landing-order.txt` (27). P3: convert twice on `2186654f7:tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.wat` byte-identical; `--check` 0; `fmt-head-fqdn-to-clojure` SCOPE does not match corpus | **one-param-spec reads TWO stdin path vectors** — convert.sh sends `["$WORK"]\n["$WORK"]\n`. Naive `git diff --name-status` yields 28; `variant-vector-to-tagged-map` was `git rm`'d; chain-order keeps files that still exist at HEAD → 27 |
| 1 | `2186654f7` → `964e82c3e` | .rs + NEW .wat | ~2 min + convert 29 s | `src/rete/kernel/tests.rs`; NEW `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.{rs,wat}` | tests.rs auto-merged (vocare comments). New .wat = `convert.sh C` | `the_join_fires_exactly_once`; `each_side_contributes_its_own_non_join_binding`; `native_agrees_with_the_oracle` PASS | First split was wrong: cherry-pick `--no-commit` left probes staged, tooling swallowed them. `git reset --soft 6a7ead983` of the two unpushed commits, recommit scripts-only then #1. Convert: `PersistentMap/get`→`map::get`, `PersistentVector/concat`→`vector::concat` |
| 2 | `8d73e74b9` → `fccc70376` | .rs | ~1.5 min | `src/rete/where_tree.rs` | auto-merged. `classify_constraint_head` ONE DOOR | `where_tree::tests` 7/7 PASS | — |
| 3 | `084e68192` → `01d4e7e0d` | .rs conflicts | ~5 min | `expr_ir.rs` `matcher.rs` `purity.rs` `validate.rs` | **3 conflicts** (matcher auto-merged). Did **not** take either side wholesale. Grok's `enum_variant_ctor` / `aggregate_field_names` as the one reader; call `wat_reader::identifier::decompose_variant` (dot) instead of `rsplit_once("::")`; HEAD 296 M tagged-map ctor + `rete_enum_unit_arg_count` kept in expr_ir; HEAD unit-arg count kept in validate; `EnumVariant` import restored (still used) | 20/20 PASS: purity, matcher, expr_ir, `probe_constructor_meta_surface_audit`, `probe_arc278_accessor_purity` | Re-express grok's registry read on main's door. On this tree variants are DOT |
| 4 | `fe301757d` → `85c7e1b09` | .rs | ~1.5 min | `src/rete/kernel/arm.rs` | auto-merged (`derive_indices`) | `rete::kernel::` 91/91 PASS | — |
| 5 | `051bc9c5b` → `562e3b2e1` | .rs | ~1.5 min + floor 215 s | `kernel/arm.rs` + `fire/mod.rs` `hash_join.rs` `rules.rs` | auto-merged | `rete::kernel::` 91/91 PASS | Checkpoint: clippy 0; floor **5394 passed / 22 skipped**, `.floor/2026-09-13T02-23-56Z/` |
| 6 | `e0df98193` → `89578f502` | .rs | 1 m 29 s (02:29:48–02:31:17Z) | `tests/rete/wat_scripts_grid_axes_live.rs` | none | `wat_scripts_grid_axes_live::grid_axes_run_and_derive_nonvacuously` PASS [12.037s] | — |
| 7 | `15dcca1df` → `b2143aaca` | NEW .wat | 2 m 31 s (02:31:24–02:33:55Z); convert 28.98 s | NEW `tests/rete/probe_arc278_stratified_query_replay.{rs,wat}` | none. New .wat = `convert.sh C` | 4/4 PASS: `native_agrees_with_the_oracle_on_every_query_shape`; `a_leading_exists_reads_correctly_through_the_replay`; `the_in_place_harvest_still_agrees`; `a_join_query_survives_the_stratified_replay` | Convert: `PersistentVector/concat`→`vector::concat`; `rete::core::i64::=`→`rete::i64::=`. `--check` 0. Same two substitutions as #1 |
| 8 | `2615e94a5` → `54c6ffbbb` | docs | 19 s (02:34:02–02:34:21Z) | 3 docs | plain `git cherry-pick -x`; auto-merged `COMPACTION-AMNESIA-RECOVERY.md` | none (docs) | Cherry-pick kept original subject; **amended** to `REPLAY(grok-rete #8): …` so P4's log shape holds. Mechanism stays plain cherry-pick (`-x` trailer kept) |
| 9 | `afb58d422` → `2c4059515` | .rs + .wat 3-way | ~6 min including 4 converts (cache ×2 ~51 s each; 7exists ×2 ~25 s each); cherry-pick 02:38:50–02:41:43Z | 21 files | cherry-pick auto-merged everything. **Still ran merge-file.** `wat/cache.wat`: convert(C^)==HEAD (composition identity); grok delta is header comments; merge-file == cherry-pick; 0 conflicts. `7exists.wat`: convert(C^)==HEAD==C^ (chain identity); convert(C)==C; merge-file == cherry-pick; 0 conflicts. `.rs`: grok Or-hoist in `compiled_cond.rs` kept; `QueryMemory` alias in `fire/mod.rs` kept; comments kept. Did not take a side wholesale | 13/13 PASS: `no_unknown_sequi_rune` ×4; `probe_arc278_7exists_native_differential` ×8; `grid_axes_run_and_derive_nonvacuously` | **stdlib `--check` double-load:** `wat --check wat/cache.wat` DuplicateDefine is pre-existing on HEAD (stdlib already loaded). Gate is boot: `--check` of 7exists loaded the new cache.wat, rc=0. Convert of cache.wat ~51 s (dense) vs ~29 s (probe) |
| 10 | `26a0d937a` → `5c51f9a8a` | .wat 3-way | 2 m 32 s (convert 51 s + cherry-pick 02:43:09–02:43:53Z) | `src/rust_deps/cache.rs`, `wat/cache.wat`, one doc | convert(C^)==HEAD; grok delta is provenance comments; merge-file == cherry-pick; 0 conflicts | none added/changed (comment-only). Stdlib boots (`--check` 7exists 0) | Comment-only follow-up to #9. Same 3-way recipe |

## Checkpoints

| at | floor | clippy |
|---|---|---|
| #5 `562e3b2e1` | `.floor/2026-09-13T02-23-56Z/` Summary [214.803s] **5394 passed, 22 skipped**, exit=0 | 0 |
| #10 `5c51f9a8a` | `.floor/2026-09-13T02-44-44Z/` Summary [219.530s] **5404 passed, 22 skipped**, exit=0 | `cargo clippy --release --all-targets -- -D warnings` CLIPPY_RC=0 (11 s) |

5404 − 5394 = 10: #7's 4 probes + #9's 4 sequi lints + #9's 2 new 7exists tests.

## Per-kind averages (this pilot, excluding checkpoint floors)

| kind | N | samples | average |
|---|---|---|---|
| docs cherry-pick | 1 | #8 19 s | **19 s** |
| .rs auto-merge | 4 | #2 #4 #5 #6 ≈ 90 s | **~1.5 min** |
| .rs conflict (re-express) | 1 | #3 ≈ 5 min | **~5 min** |
| NEW .wat (convert + tests) | 2 | #1 ~2 min + 29 s convert; #7 2 m 31 s | **~2.5 min** |
| .wat 3-way, convert identity on delta | 2 | #10 2.5 min; #9 6 min (21 files + 4 converts) | **~4 min** |

Convert cost: **~29 s / probe file**, **~51 s / `wat/cache.wat`**. Dominated by `positional-ctor-to-map`. Never two wat runs.

## Extrapolation to 651

Census (SEAM): 651 linear, **376 docs-only**, **275 touch code**, **113 of those touch a file main also changed**.

10 done → **641 remain** ≈ 375 docs + 266 code, of which ~103 still share a file with main.

| class | n remaining | unit cost | subtotal |
|---|---|---|---|
| docs-only | ~375 | 20 s | **~2.1 h** |
| grok-only code (clean cherry-pick / convert) | ~163 | 2.5 min | **~6.8 h** |
| shared file (3-way or re-express) | ~103 | 4 min (mix of #3/#9/#10) | **~6.9 h** |
| floors at stone checkpoints | say 10 | ~4 min | **~0.7 h** |
| **total remaining** | | | **~16.5 h** |

Optimistic if most shared files auto-merge like #2/#4/#9 (not #3): shared unit ~2 min → **~13 h**. Pessimistic if many #3-class re-expresses at 5–8 min: **~20 h**.

The brief's 4–8 h for *this* 10-commit pilot was high: #3 did not need a third behaviour, and both `wat/cache.wat` 3-ways were convert(C^)==HEAD with a comment-only grok delta. The 651-scale cost is the remaining shared `.wat` files that are **not** identity.

## Tricks catalogue (reusable)

1. **`one-param-spec.wat` reads two path vectors.** convert.sh special-cases that stem.
2. **TIME ONE FILE FIRST.** convert ~29 s probe / ~51 s dense stdlib. A 1489-file chain is hours.
3. **Never two wat processes.** `ps` before and after.
4. **Chain-order derives, then filters to files that still exist at HEAD.** A `git rm`'d identity-codemod is not a step.
5. **One PROVISIONAL override:** `bare-variant-to-qualified` BEFORE `positional-ctor-to-map`. Do **not** treat as settled. Do **not** add a second MOVE (assertion-failed before match-arm) unless convert hits STOP-2; this pilot did not.
6. **Ownership on `.rs` conflict:** main's syntax + non-rete behaviour; grok's rete behaviour re-expressed. Crib is `git show merge/grok-rete:…/SCORE-merge-grok-rete.md` only — never check that branch out. #3: `decompose_variant` (dot) not `rsplit_once("::")`; keep HEAD map ctor.
7. **Cherry-pick `--no-commit` stages everything.** Commit tooling first, then the replayed files, or the tooling commit swallows the step.
8. **New `.wat` = convert.sh stdout, never hand-edit (STOP-2 / R21).**
9. **Modified `.wat` = `git merge-file working convert(C^) convert(C)`.** Working copy is HEAD (main's syntax). Save HEAD before cherry-pick; cherry-pick of unconverted grok onto converted HEAD can look clean and still be the wrong recipe — compare to merge-file.
10. **When convert(C^)==HEAD, merge-file == applying convert(C)-convert(C^) onto HEAD.** If the grok delta has no convertible forms, cherry-pick == merge-file. Still run merge-file; log the identity.
11. **`wat --check wat/cache.wat` DuplicateDefine is stdlib double-load**, pre-existing. Prove the file by booting it as stdlib (`--check` of some other corpus file).
12. **Docs-only: `git cherry-pick -x`, then amend the subject to `REPLAY(grok-rete #N):`** so the log is grepable. Keep the trailer.
13. **Leave the orchestrator's unstaged SEAM.md composition finding uncommitted** in replay commits.
14. **Embedded wat in `.rs` strings is by hand.** None in #1–#10 needed it.

## STOP triggers

None fired.

SEAM composition finding 2 (chain STOPS at match-arm because `try-type-of` evaluates positional `assertion-failed!` that a later step migrates) was observed as unstaged SEAM prose mid-pilot and **not applied**. This pilot's converted files did not contain positional `assertion-failed!`. If a later convert hits STOP-2 on that form, that is the finding.
