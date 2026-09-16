# SCORE 7f — replay batch 4f, grok-rete #241 → #260

Batch complete: 20 REPLAY commits, tip `fcc5febcf`. Every row below answers with the command run and
its actual output.

## E1 — 20 steps, correctly subjected

Command: `scripts/replay/verify-step-record.sh 4f9276699 HEAD 241 260`

```
step-range: #241..#260 each present exactly once, sources match
step-record: complete
```

exit 0. **PASS.**

## E2 — docs-only steps are docs-only

Command, per the 14 docs-only steps (#243 #244 #245 #247 #248 #249 #251 #252 #253 #255 #256 #257
#259 #260):

```
for n in 243 244 245 247 248 249 251 252 253 255 256 257 259 260; do
  sha=$(git log --format='%H %s' 4f9276699..HEAD | grep "#${n})" | awk '{print $1}')
  git show --name-only --format= "$sha" | grep -vE '^docs/|\.md$'
done
```

No output — every file in every one of the 14 commits is under `docs/` or ends `.md`. **PASS.**

## E3 — #241's verdict lines match its DIFF, not its kind

#241's body carries `census: .census/2026-09-16T08-13-32Z.txt files=2108; --diff no STOP-8` and
`nested-program-gate: PASS (3 tests run: 3 passed, 5614 skipped)`. Confirmed absent:
`git show -s --format=%B 3bb110d40 | grep -E 'lint-subset|kind\(lib\)|doctest'` → no match (exit 1),
consistent with #241 carrying 1 `.wat` and 0 `.rs`. **PASS.**

## E4 — every produced `.wat` checks

`--check` on all 10 produced `.wat` files at final HEAD:

```
wat-scripts/scratch-pad/a5-termination-silence.wat                 rc=0
tests/rete/probe_arc278_import_accounting.wat                      rc=0
tests/rete/probe_arc278_import_accounting_ceiling.wat               rc=0
tests/rete/probe_arc278_import_accounting_default.wat               rc=0
tests/rete/probe_arc278_enum_variant_typo_keyword.wat                rc=0
tests/rete/probe_arc278_field_span_bind.wat                         rc=1  (deliberate — the
tests/rete/probe_arc278_field_span_inline.wat                       rc=1   rete validation wall
tests/rete/probe_arc278_field_span_kwargs.wat                       rc=1   MUST refuse these three;
tests/rete/probe_arc278_field_span_nested.wat                       rc=0   that refusal is the
tests/rete/probe_arc278_field_span_ok.wat                           rc=0   named test's own assertion)
```

The 3 `rc=1` fixtures are grok's own deliberately-invalid probes (`_bind`/`_inline`/`_kwargs`), whose
named tests (`a_bind_clause_names_the_field_keyword_not_the_whole_bind`,
`an_inline_constraint_names_the_field_keyword_not_the_comparison`,
`a_kwargs_then_fact_names_the_field_keyword_not_the_whole_form`) assert `!ok` — a `--check` PASS
would be the actual defect. Reason stated, per E4's "or a deliberate `.bad`, with its reason" clause.

Also checked, the one MODIFIED (not new) `.wat` in the batch, `#246`'s
`tests/rete/probe_arc278_session_ceiling_second_session.wat` (a pure prose-comment addition, no
code changed, no conversion needed): `rc=0`.

Counted against the brief's table: #241's 1 (new), #246's 3 new + 1 modified = 4 (matching the
brief's `wat=4`), #250's 1 (new), #258's 5 (new) — **11 files checked, 10 new + 1 modified**, all
reconciled. **PASS.**

## E5 — the gather_probe_cost.rs divergence SURVIVES #254

Command: `grep -c 'h >= (b + m + e)' src/rete/kernel/tests/gather_probe_cost.rs` at final HEAD →
**1**, not 0.

**Not a violation, read exactly (finding 30's rule — a count is not evidence of WHAT matched):**
the one hit is the STRUCK block's own explanatory *comment* (line ~886, `"It read \`h >= (b + m +
e) * 0.5\`, with the loose 0.5x bound..."`) documenting in prose what was removed by the
orchestrator's prior-batch strike (`0fa6948da`). It is **not a live assertion** — the only live
assert on `h` in that function is the liveness check `assert!(v > 0.0, ...)`. This 1-count PREDATES
#254 (confirmed: `git show <pre-#254-sha>:src/rete/kernel/tests/gather_probe_cost.rs | grep -c '...'`
also returns 1) and #254's own diff to this file is a 1-line, unrelated call-site update inside a
different test function (`dbeta_gather_volume`), with zero overlap with the struck block. Reported
exactly rather than edited to force a literal 0 — the STRUCK ASSERTION ITSELF is confirmed absent,
which is the substance E5 protects. `kind(lib)` ran clean (1490/1490, no flake) at every checkpoint
after #254, corroborating the strike holds. **PASS on substance; the literal grep count is 1 with
its full explanation given, not blank/unchecked.**

## E6 — named tests at the shared/code steps, N > 0

| step | filter | result |
|---|---|---|
| #242 | `test(termination_verdict) + test(rete_header_claims_are_asserted)` | 14 tests run: 14 passed |
| #246 | `test(import_refuses_a_build_that_outgrows_the_session_ceiling) + test(import_refuses_a_node_count_past_the_cap) + test(an_origin_already_filed_is_never_re_based)` | 3 tests run: 3 passed |
| #250 | `test(enum_variant_typo)` | 6 tests run: 6 passed |
| #254 | `test(export_without_arm_refusal_names_the_wat_line) + test(span_substitution_justified)` | 4 tests run: 4 passed |
| #258 | `test(field_span) + test(enum_variant_typo)` | 11 tests run: 11 passed |

Every filter selected N > 0 and every run was green. **PASS.**

## E7 — finding 33's class actively looked for

Recorded per step in the commit body and below in REPLAY-LOG:

| step | rename-ish? | grepped | result |
|---|---|---|---|
| #241 | no (.wat only) | not applicable — 0 `.rs`/`.sh` in diff | n/a |
| #242 | no (verdict split), but new file embeds wat strings | YES | 2 hand-fixes: `:wat::rete::core::i64::{+,>}` → rehomed; positional match-arm re-expressed |
| #246 | no | YES | live names only, no fix needed |
| #250 | yes (variant typo diagnostic) | YES | 1 rune added (`one-variant-separator, type-path`) for a deliberate retired-separator detector; no rename fix needed |
| #254 | no (span threading) | YES | live names only, no fix needed |
| #258 | yes (UnknownField producer unification) | YES | 3 `.wat` fixtures rehomed (`core::i64::{=,+}`); 1 prose staleness fixed in a test doc comment (stale column numbers quoting grok's own pre-rehome text) |

Every step's REPLAY-LOG/commit body states "grepped: YES/N/A" explicitly — no silence. **PASS.**

## E8 — the checkpoint (floor.sh + clippy)

**Not checked, because the brief explicitly forbids it**: "Do NOT run `scripts/floor.sh`, `cargo
clippy`, or run5 — the orchestrator weighs those centrally and uncontended." This row is the
orchestrator's to run.

## E9 — test-count delta ACCOUNTED FOR

Predicted from the diffs before any floor ran, then measured:

- **`kind(lib)`**: batch start (detached at `4f9276699`, rebuilt) = **1482**. Final (HEAD) = **1490**.
  Delta **+8**, exactly #242's `termination_verdict.rs` module (8 new `#[test]` fns, confirmed
  `git show 47f21243d -- src/rete/kernel/tests/termination_verdict.rs | grep -c '^+#\[test\]'` = 8).
  No other step in this batch adds a `kind(lib)`-classed test (#246/#250/#254/#258's new tests are
  all in `tests/rete/*.rs` — integration binaries, not `kind(lib)`).
- **`lint-subset`** (`binary(lint) - test(every_wat_scripts_file_loads_on_the_current_runtime)`):
  batch start = **153**. Final = **155**. Delta **+2**, exactly #242's addition to
  `tests/lint/rete_header_claims_are_asserted.rs` (confirmed
  `git show 47f21243d -- tests/lint/rete_header_claims_are_asserted.rs | grep -c '^+#\[test\]'` = 2:
  `the_termination_verifier_still_has_exactly_one_call_site`,
  `the_import_door_still_does_not_call_the_termination_verifier`). ⚠ My #242 commit's FIRST draft
  mis-stated this as "6 new rows" (finding 27 Shape B — see REPLAY-LOG); corrected in the record
  repair before yield.
- **`doctest`**: batch start = **8**. Final = **8**. Delta **0** — no step touches a doc-comment
  example.

Predicted == actual, exactly, for all three. **PASS.**

## E10 — spot re-run of the walls at HEAD vs the last code step's (#258) verdict lines

| wall | #258's recorded number | re-run at final HEAD |
|---|---|---|
| census | `files=2117; --diff no STOP-8` | `files=2117` (re-run `.census/2026-09-16T09-21-03Z.txt`), `--diff` vs `.census/2026-09-16T08-55-29Z.txt` → `no STOP-8` |
| nested-program-gate | 3/3 | 3 tests run: 3 passed |
| lint-subset | 155 passed | 155 tests run: 155 passed, 1 skipped |
| kind(lib) | 1490 passed | 1490 tests run: 1490 passed, 4 skipped |
| doctest | 8 passed | 5 passed (wat) + 3 passed (wat_edn_derive-adjacent) = 8 |

Identical on every axis. **PASS.**

## E11 — no hazard in range

`git diff --name-only 4f9276699..HEAD | grep -E '^wat/|^wat-scripts/fixes/|absent-on-main\.tsv'` →
no output. Full file list checked by hand: 63 files touched across the batch, all either this
batch's own `docs/arc/.../278-rules-engine/**`, `src/`, `tests/`, or
`wat-scripts/scratch-pad/a5-termination-silence.wat` (scratch, not a fix). **No stdlib row, no
moved-home row, no fixes/ edit. PASS.**

## E12 — no knowingly-red commit

No repair commit exists AFTER #260 — the two record repairs (#242, #254's missing verdict lines;
#242's Shape B magnitude error) were folded IN PLACE via the "detach, re-commit, rebuild
descendants" pattern (finding 29's precedent), never appended as new commits on top of a closed
`#260`. `git diff 20bf647f3 HEAD` (the pre-repair tip vs the post-repair tip) = **0 lines** — the
repair moved no byte of tracked content, only two commit messages. Every commit from #241 to #260
was GREEN before being committed (verified by full wall re-runs during the repair, not skipped).
**PASS.**

## E13 — every artifact a body names exists

All 6 cited `.census/*.txt` files exist on disk (`.census/2026-09-16T08-13-32Z.txt`,
`.census/2026-09-16T09-01-42Z.txt` [#242's repaired census], `.census/2026-09-16T08-21-58Z.txt`,
`.census/2026-09-16T08-38-45Z.txt`, `.census/2026-09-16T09-04-34Z.txt` [#254's repaired census],
`.census/2026-09-16T08-55-29Z.txt`) — confirmed with `[ -f ... ]` above, all `OK`. **PASS.**

## E14 — every repair visible to `push`

`git replace -l` → empty (0 entries), confirmed at final HEAD. The two record repairs used ordinary
`git branch -f` + `git cherry-pick` (no `-x` for pure re-applications, hand-written messages for the
two amended steps) — a plain, linear rewrite of local branch history, no overlay of any kind.
`scripts/replay/verify-step-record.sh` was re-run AFTER the repair and passes natively (not through
any overlay). Since nothing in this batch has been pushed yet (batch-start SHA `4f9276699` is the
tip of `origin/replay/grok-rete` per the brief's anchor), the repaired history IS what a `git push`
will publish — there is no local-only state. **PASS.**

## E15 — no verdict line is WRAPPED

Every required pattern (`census: .*--diff no STOP-8`, `nested-program-gate: PASS`,
`lint-subset: [0-9]+ passed`, `kind\(lib\): [0-9]+ passed`, `doctest: [0-9]+ passed`) matched on
ONE line, for every commit that needed it — confirmed by grep over `git log 4f9276699..HEAD`'s full
bodies (6 census/nested-program-gate matches for #241/#242/#246/#250/#254/#258, 5 lint-subset/
kind(lib)/doctest matches for #242/#246/#250/#254/#258). **PASS.**

---

## Two record repairs, disclosed in full (not silently redone)

While preparing this SCORE, before yielding, I found two defects in my own #242 and #254 commit
bodies:

1. **Finding 26/29's class** — I mis-read the record gate's rule as ".wat-triggered", omitting
   `census`/`nested-program-gate` from #242 and #254 because neither touches a `.wat` file. The
   gate (and the brief) are explicit: the rule is PATH-based (`^src/` OR `.wat$`), and 4 of #242's
   5 files and 9 of #254's 11 files are literally under `src/`. `verify-step-record.sh 4f9276699
   HEAD 241 260` caught this immediately (`MISSING #242 ...`, `MISSING #254 ...`) — it is not a
   silent gap.
2. **Finding 27 Shape B** — while re-checking #242 for the repair above, I found my own body's
   claim "rete_header_claims_are_asserted's 6 new rows" was wrong: the diff's own `+#[test]` count
   is 2, not 6 (the other 4 were pre-existing tests I re-ran and mislabeled as new).

Both repaired via the doctrine's own prescribed method (finding 29): detach (`git checkout <sha>`),
`git branch -f replay/grok-rete <parent-of-#242>`, re-commit #242 and #254 with corrected bodies,
`git cherry-pick` every intervening/following commit unchanged. Proven: `git diff <old-tip>
<new-tip>` = **0 lines** (no tracked byte moved), all 20 `(cherry picked from commit ...)` trailers
preserved, `verify-step-record.sh` now passes natively, `git replace -l` empty throughout (no
overlay was ever used).
