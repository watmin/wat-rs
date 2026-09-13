# SCORE 1 — the PILOT: grok-rete #1–#10 onto main's syntax

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
HEAD: `5c51f9a8a` REPLAY(grok-rete #10).
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
floor #5  scripts/floor.sh   .floor/2026-09-13T02-23-56Z
          Summary [ 214.803s] 5394 tests run: 5394 passed, 22 skipped   exit=0
floor #10 scripts/floor.sh   .floor/2026-09-13T02-44-44Z
          Summary [ 219.530s] 5404 tests run: 5404 passed, 22 skipped   exit=0
clippy    cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

No STOP fired. Yield after #10.

## EXPECTATIONS

| # | result |
|---|---|
| P1 | **identical, 27 lines.** `scripts/replay/chain-order.sh de827fb4c a3218644d --no-overrides` == `bootstrap/landing-order.txt`. Re-verified at SCORE. Naive `git diff --name-status … \| grep ^A` is 28 because `variant-vector-to-tagged-map.wat` was added then `git rm`'d; chain-order keeps files that still exist at HEAD. |
| P2 | **one MOVE, marked PROVISIONAL.** `scripts/replay/chain-order.overrides`: `bare-variant-to-qualified` BEFORE `positional-ctor-to-map`, reason `49f03f179`. With override: bare-variant line 25, positional-ctor line 26, still 27 lines. **Not treated as settled.** A second MOVE (assertion-failed before match-arm, SEAM composition finding 2) was **not** added; the pilot brief names one override. |
| P3 | **byte-identical; out-of-scope untouched.** Two runs of `convert.sh 2186654f7 tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.wat` identical; `--check` 0. `fmt-head-fqdn-to-clojure` SCOPE is `wat-scripts/fmt/rules/*.wat wat-scripts/grep/*.wat` — a corpus path is skipped. |
| P4 | **ten `REPLAY(grok-rete #1..#10)` plus tooling.** Hashes below. Each body carries the source hash and `(cherry picked from commit …)`. #8 was a plain `git cherry-pick -x`; subject amended to the REPLAY shape. |
| P5 | **all pass at their own step, named in PILOT-LOG.** #1 3; #2 7; #3 20; #4/#5 kernel 91; #6 `grid_axes_run_and_derive_nonvacuously`; #7 4; #8 docs (none); #9 13; #10 comment-only (none). PASS lines read, not inferred. |
| P6 | **clean.** Every `.wat` the pilot produced `--check` 0, except `wat/cache.wat` standalone DuplicateDefine which is **pre-existing on HEAD** (stdlib double-load). Boot as stdlib: `--check tests/rete/probe_arc278_7exists_native_differential.wat` rc=0 with the new cache.wat on disk. No `.bad`. No hand-edit of converted `.wat` (R21). |
| P7 | **logged.** Both #9 and #10: convert(C^)==HEAD; grok delta is comments; `git merge-file` rc=0, 0 conflict markers; merge-file result **byte-identical** to the cherry-pick. Both sides' intent present: main's syntax (already at HEAD) + grok's comment change. |
| P8 | **0 failed · 0 clippy lines.** #5: 5394/5394. #10: 5404/5404. Clippy 0. |
| P9 | **PILOT-LOG table + per-kind averages + ~16.5 h remaining.** Docs 19 s; .rs auto-merge ~1.5 min; .rs conflict ~5 min; new .wat ~2.5 min; .wat 3-way ~4 min. Convert 29 s/probe, 51 s/`cache.wat`. Pilot itself was ~1–1.5 h, not 4–8 h: #3 needed no third behaviour; both cache.wat 3-ways were identity-on-code. |
| P10 | **14 tricks in PILOT-LOG.** one-param-spec two vectors; TIME ONE FILE; never two wat; HEAD-exists filter; provisional override; ownership re-express; staged-file split; convert never hand-edit; merge-file recipe; identity still run; stdlib `--check` double-load; docs amend; SEAM unstaged; no embedded-wat in #1–#10. |

## The ten

| # | source | replay | kind |
|---|---|---|---|
| tooling | — | `419350022` | `scripts/replay/` |
| 1 | `2186654f7` | `964e82c3e` | .rs + NEW .wat |
| 2 | `8d73e74b9` | `fccc70376` | .rs |
| 3 | `084e68192` | `01d4e7e0d` | .rs conflicts (re-expressed) |
| 4 | `fe301757d` | `85c7e1b09` | .rs |
| 5 | `051bc9c5b` | `562e3b2e1` | .rs + floor/clippy |
| 6 | `e0df98193` | `89578f502` | .rs |
| 7 | `15dcca1df` | `b2143aaca` | NEW .wat |
| 8 | `2615e94a5` | `54c6ffbbb` | docs, plain cherry-pick |
| 9 | `afb58d422` | `2c4059515` | .rs + `wat/cache.wat` 3-way |
| 10 | `26a0d937a` | `5c51f9a8a` | `wat/cache.wat` 3-way + floor/clippy |

## #3 — the one re-express (not wholesale)

Conflicts in `expr_ir.rs`, `purity.rs`, `validate.rs` (matcher auto-merged).

- Grok's `enum_variant_ctor` / `aggregate_field_names` stay the one registry reader.
- Call `wat_reader::identifier::decompose_variant` (dot) instead of grok's `rsplit_once("::")`.
- HEAD 296 M tagged-map ctor + `rete_enum_unit_arg_count` kept in expr_ir.
- HEAD unit-arg count kept in validate; `EnumVariant` import restored (still used).

No STOP-3. Tests 20/20 at the step.

## `wat/cache.wat` 3-way (#9 and #10)

```
HEAD  == main's syntax of the merge-base file
C^    == grok-rete parent
convert(C^) == HEAD     composition identity on this file
convert(C)  == convert(C^) + grok's comment hunk
git merge-file working=HEAD base=convert(C^) theirs=convert(C)   rc=0, 0 conflicts
result == cherry-pick
```

#9 grok hunk: failure-surface comments (open decision / exigere). #10 grok hunk: study-oracle citations marked GONE / provenance, not rete `$oracle`.

Standalone `wat --check wat/cache.wat` prints DuplicateDefine on HEAD too — the file is already in the stdlib load set. Not STOP-2.

## Trap doors, not walked

- Did not take one side of an `.rs` conflict wholesale (#3).
- Did not hand-edit a converted `.wat`.
- Named each test and read its PASS line.
- Did not treat the provisional override as settled; did not add a second MOVE.

## Blast radius

Replayed commits' own files, `scripts/replay/`, this directory's PILOT-LOG and SCORE. `wat-scripts/fixes/` untouched. `origin/main` untouched. Not pushed.
