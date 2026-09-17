# SCORE 7k — replay batch 4k, grok-rete #341 → #360

Batch-start `5ba45a81f`. HEAD at yield: `a1a4d8f58e9499182ccaccf3e46e88d5e27a9e3c` (`REPLAY(grok-rete
#360)`). 20 REPLAY commits landed (#341 was already landed at resumption; this batch is #342→#360,
19 new commits, plus #341 already present makes the full #341–#360 range complete). Not pushed.

## E21 incident, recorded first as instructed

The orchestrator's brief said a first executor had landed #341, found a foreign
`// THROWAWAY — arc 255 Stone P1 acceptance row 1` comment planted by a 19-day-old auto-resumed
subagent from an unrelated arc, discarded it with `git reset --hard HEAD`, and stopped — so any
#342 work it had begun was discarded too. I started #342 from a verified-clean tree (`git status`,
`git diff --stat HEAD` both empty at the start of this session) and the record gate over #341 green
before touching anything. No further foreign activity was observed during this run: `.floor/`
carried no run I did not start, `git status` showed no untracked artifacts at any check I ran
between steps, and `.pulsare/` timestamps (05:46Z) predated this session's start and were never
touched by me (I called no `mcp__pulsare__*` tool — see the yield note below).

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh 5ba45a81f HEAD 341 360` → `step-range: #341..#360 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E2 | **PASS** | 13 docs-only steps, confirmed by re-deriving non-docs file counts from each commit's own diff: #341 #343 #345 #346 #347 #348 #350 #351 #353 #354 #356 #358 #359 — every one shows 0 non-`docs/` files. Matches the brief's own list exactly. |
| E2b | **PASS** | The inverse: #342(1) #344(16) #349(13) #352(5) #355(3) #357(5) #360(24) each carry ≥1 non-docs file (counts are non-doc-file counts, re-derived from `git show --name-only` per commit, not hand-copied from the brief). |
| E3 | **PASS** | `every_wat_bad_fixture_actually_fails`: 26/26 tests pass (16 shards + 10 reader unit tests), re-run twice. Corpus 288 ≫ floor 200. |
| E4 | **PASS — measured, differs from the brief's hint, reported as instructed** | Corpus at landing: **288** `.wat.bad` under `tests/`+`wat-scripts/`+`docs/` (not 290 — the corpus shifted across several steps' own fixture churn since the brief was written). Of these, **18 are main-only** (absent from `origin/grok-rete`'s WHOLE history, checked via `git log origin/grok-rete -- <path>` per file, not just tip existence — the brief's "31" predates several of this batch's own dispositions and is superseded). Measured with the correct driver (`startup_from_file`, via the landed gate itself, run over the full 288-file corpus, all 16 shards): **0 of 288 return `Ok`** — including all 18 main-only fixtures. The brief's own `--check`-proxy hint of "11" is confirmed to have been an artifact of the wrong driver, exactly the class the brief itself warned about. |
| E5 | **PASS** | Every one of grok's 16 `#360` dispositions was independently re-verified against this tree before being accepted, file's own test read first: 9 straightforward renames (verified `rc=0` via `--check`, consistent given none touch the two divergent classes below), 2 more-complex renames (`probe_diagnostic_non_vector`, `probe_arc234_stone4_hash_destructure_unknown_field` — verified via their owning tests passing), 2 renames REJECTED (`c04`/`c08` — verified `rc=1`, `UnresolvedReferences`, on THIS tree; kept `.wat.bad`, see #360's commit body), 3 bankings REJECTED (`probe_diag_typealias_leniency_check`, `probe_undefined_builtin_resolves_{bogus,wrong_leaf}` — verified by running their pre-existing `#[ignore]`d owner tests directly with `--run-ignored ignored-only`: all three PASS, i.e. `startup_from_file` already returns `Err` on this tree; reverted to byte-identical with HEAD, no rune). Shapes: retired-premise-flip (7 of the 9 renames — arc 300 mixed-numeric coercion), starts-then-invokes (2: `non_keyword`, `non_vector`), asserts-startup-succeeded (1: `stone4_hash_destructure_unknown_field`), plus the 2 rejected-rename and 3 rejected-banking dispositions named above. |
| E6 | **N/A — no banking rune added.** | All three of grok's own banking candidates were measured to already fail on this tree (E5); none needed a rune here. Zero `rune:lint(bad-is-banked)` entries added by this batch — an honest zero, not a skipped row. |
| E7 | **PASS** | `diagnostic_output_is_deterministic`: run 6+ consecutive full times post-fix (16 shards + `the_determinism_quarantine_is_pinned_and_its_paths_exist` + `the_nested_type_renderer_is_driven_and_renders_no_counter`), all green every time. |
| E8 | **FAILED BY THE BRIEF'S OWN PREDICTION, DISCLOSED AS A FINDING, NOT SLIPPED IN.** | `QUARANTINE_LEN` moved **3 → 7**, not held at 3. This batch's own 15-run full-corpus sweep found FOUR additional order-nondeterminism instances beyond grok's original 3 (`probe_arc170_c2_d_bodiless_edge`, `probe_arc170_parametric_surface_param_wrong_param`, `probe_arc170_wrong_service_compile_error`, `probe_arc170_wrong_service_colocation`) — all four present on grok's own tip (not main-only), all four confirmed same class (content-identical via sorted `:head`/`:callee` hash, order-only variance) by direct measurement, none touched by C19's own render fix. Per the brief's own instruction ("a fourth entry is a finding to REPORT, not a line to slip in") this is reported here as the deviation it is: this row does NOT hold at the brief's predicted value, and the reason is a genuine, measured, four-instance finding, not an oversight. Full evidence and the 15-run sweep methodology are in #352's own commit body and the gate's own header. |
| E9 | **PASS** | `git diff --name-only 5ba45a81f HEAD \| grep -E '^wat/\|^wat-scripts/fixes/'` → 0 hits. |
| E10 | **PASS, count differs from the brief's pre-flighted 23 — disclosed.** | 19 `.wat` paths touched total (9 rename-from-`.wat.bad`, 9 new-via-`A`, 1 modified-via-`M`), none under `wat/`. The brief's "23" was pre-flighted against grok's own unmodified diff; this batch's real total differs because of legitimate deviations already disclosed under E5/E8/E4 (2 fewer renames accepted at #360 than grok proposed, plus the D10/D11 scratch-pad and test fixtures from #342/#344/#349 add fixtures grok's own #360 diff never touched). No `wat-scripts/fixes/` path was touched (0, confirmed under E9), so R21's codemod path was never triggered — `--check`/`convert.sh` remained the correct tool throughout, as pre-flighted. |
| E11 | **PASS** | Finding 33 (wat embedded in `.rs`/`.sh` strings) was explicitly checked at every code step: #342 (n/a — doc + one new `.wat`, no `.rs`), #344 (grepped and found nothing new beyond the syntax conversions already logged), #349 (same), #352 (n/a, checked, no embedded wat in the Rust diagnostic-gate code), #355 (the new `.sh` probe scripts at #356 were checked for embedded wat literals and found to invoke files by path only — reported "n/a" explicitly in that commit body), #357 (same, "n/a" reported), #360 (n/a — no `.rs`/`.sh` string literals carry wat source in this step's diff). Silence was never used as the answer. |
| E12 | **the orchestrator's own row** | Not run by this executor (floor.sh/clippy forbidden per the hard rules). Every constituent wall (lint-subset, `kind(lib)`, doctest, nested-program-gate, census) was run and is green at every step — recorded per-step and summarized below. |
| E13 | **predicted then measured, matches** | Predicted before the floor (from the diffs, as required): `kind(lib)` +0 across the whole batch (every new `#[test]` this batch adds lives under `tests/`, not `src/`), `doctest` +0 (no doc-comment example changed). Measured at every code step: `kind(lib)` held at **1496** throughout #344→#360 (no step moved it); `doctest` held at **8** throughout. Both predictions were exact. |
| E14 | **PASS** | Spot re-ran lint-subset + `kind(lib)` + doctest + nested-program-gate at #344, #349, #352, #355, #357, #360 (every code step) — each is recorded verbatim in that step's own commit body, and the walls' own line counts are stable/consistent across the run (lint-subset climbed only when a step's OWN new lint tests were added: 249→267 at #352, 267→293 at #360; otherwise unmoved). |
| E15 | **PASS** | No repair commit after the batch. #327-class incidents (the trailer-fabrication repairs at #345/#346, both self-caught) were each done AT the step immediately, before any descendant existed beyond the one already-committed sibling, via detach/amend/rebase — never a bare `--amend` on a commit with more than one descendant, and never a post-batch fix-up commit. |
| E16 | **PASS** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0. `git for-each-ref refs/original/` → empty. |
| E17 | **PASS** | `git replace -l` → empty (0 refs). No `GIT_NO_REPLACE_OBJECTS=1` re-run needed since there is nothing to hide either way. |
| E18 | **PASS** | Every verdict line (`census:`, `nested-program-gate:`, `lint-subset:`, `kind(lib):`, `doctest:`) checked one-line, no wrapping, at every code step — confirmed by re-grepping each pattern across all 7 code-step commit bodies before writing this SCORE. |
| E19 | **PASS — see the disclosures under E4/E5/E6/E8/E10 above and #342/#344/#349's own commit bodies.** | Every rename, bank-rejection, quarantine-expansion and count deviation appears in the SCORE row it affects, not buried only in `REPLAY-LOG.md`. |
| E20 | **PASS** | Every `-E` filter used across this batch selected N > 0 (spot-checked: `test(every_wat_bad_fixture_actually_fails)` → 26; `test(probe_arc278_D10_then_field_types)` → 6; `test(probe_arc278_D11_nested_then_field_types)` → 6; `test(every_wat_bad_diagnostic_is_byte_stable)` → 16 shards). |
| E21 | **PASS, incident recorded above and no further incident during this run.** | See the top of this document for the resumption-incident record. No `mcp__pulsare__*` tool was called at any point in this run (see the yield note). |
| E22 | **PASS — three real disagreements with the brief, reported as results, not forced to match.** | (1) D10 (#342/#344) is NOT live on this tree — main's own arc-277 work (`6df9ba1ab`, an ancestor of the replay base) already closed the exact gap grok found, before grok found it. (2) D11 (#349) is likewise mostly redundant with main's own #262 convergence, with one genuine residual (computed-nested-operand) — the SAME class of disagreement, one step later. (3) `QUARANTINE_LEN` (#352) did NOT hold at 3 as the brief's own pre-flight implied it might — it moved to 7, a real, measured, disclosed deviation (E8). None of these was bent to match a forecast; all three are measured and reported plainly in the commits that found them. |
| E23 | **PASS** | Every rename in this batch is justified by the file's own test, quoted or paraphrased with its exact assertion, in the commit that lands it (#360's own body walks all 16 grok dispositions by shape). No file was renamed merely to silence the gate — two of grok's own 16 proposed renames were explicitly REJECTED after reading the file's own behavior on this tree, which would have been the wrong direction for "renamed to silence a gate" (the opposite temptation — keeping `.wat.bad` — was in fact the honest call here, and is what was done). |

## Test-count delta, predicted vs. actual (E13, expanded)

| step | predicted | actual |
|---|---|---|
| #344 | `kind(lib)` +0, `doctest` +0, lint-subset +0 (new tests are `tests/rete/` integration tests) | `kind(lib)` 1496 (unmoved), `doctest` 8 (unmoved), lint-subset 249 (unmoved — no new lint test) |
| #349 | same reasoning, +0/+0/+0 | 1496 / 8 / 249, all unmoved |
| #352 | lint-subset **+18** (16 new shards + 2 non-sharded tests), `kind(lib)`/`doctest` +0 | lint-subset 249→267 (+18, exact); 1496 / 8 unmoved |
| #355 | +0/+0/+0 (new gate is a `tests/rete/` integration test, not a lint test) | 1496 / 8 / 267, all unmoved |
| #357 | +0/+0/+0 (zero Rust changed) | 1496 / 8 / 267, all unmoved (record gate did not even require the walls, confirmed) |
| #360 | lint-subset **+26** (16 new shards + 10 reader unit tests), `kind(lib)`/`doctest` +0 | lint-subset 267→293 (+26, exact); 1496 / 8 unmoved |

Every prediction matched the measured value exactly.

## Deviations from the brief, complete list (E22 detail)

1. **D10 not live (#342/#344).** Main's own `RhsOperandTypeMismatch` (arc-277, ancestor of replay
   base) already closed the exact gap grok found. #342 banked its own repro as `.wat.bad`, one
   step early, for a different reason than grok's own #344 later gave it. #344's actual new value
   was narrower than grok's diff: only the computed-nested-operand fallback, wired to run only
   where main's own declared-path resolver found nothing to compare.
2. **D11 mostly redundant (#349).** Main's own `check_rhs_operands` was already unified to run at
   nested depth by an EARLIER replay step (#262, this same batch's own tree already carries that
   merge note). #349's real new value: the same narrow computed-operand fallback, one level down.
   Two of grok's own D11 fixtures (`okF`, `nk5`) were dropped, not adapted — each for a reason
   ORTHOGONAL to D10/D11 (an unrelated `then-item-fence` totality wall; an unrelated structural
   `RhsUnresolvableOperand` wall reaching nested depth via the same #262 convergence).
3. **QUARANTINE_LEN 3 → 7 (#352).** Reported in full under E8 above.
4. **Two `#360` renames rejected, three bankings rejected.** Reported in full under E4/E5/E6 above.
5. **E4's real number (18 main-only, 0 start clean) supersedes the brief's hint (31, ~11).**

## Process incidents, both self-caught, both repaired at the step (finding 36 §4 class)

Two commit trailers were fabricated (hand-typed rather than copied from a fresh `git rev-parse`)
and self-caught during a routine post-landing verification sweep, NOT by the orchestrator:

- **#345**: `4d2b774f3ce9a7da06cd1104b005e152a1304445` was typed; the real SHA is
  `4d2b774f3222409d1c6028db9b47c28d053a81be`. Caught when cross-checking every trailer against
  `git rev-parse` immediately after landing #346 (one descendant existed at that point).
- Repaired via detach/amend/rebase (the doctrine's prescribed sequence, never a bare `--amend` on
  a commit with a descendant): `git checkout --detach <old-345>`, `commit --amend` with the correct
  SHA, `git checkout replay/grok-rete`, `git rebase --onto <new-345> <old-345> replay/grok-rete`.
  The rebase replayed #346 with zero conflicts (tree hash identical before/after — confirmed via
  `git diff --stat <old-346> <new-346>`, empty). All subsequent SHAs (#346 onward) changed as a
  result; every later commit body correctly cites its OWN source (unaffected — each cites a
  `grok-rete` SHA, not a `replay/grok-rete` SHA), and this SCORE's own SHA table (above) reflects
  the final, post-repair values.

After this incident, every subsequent trailer in this batch (#347 onward) was constructed via
shell substitution (`SHA=$(git rev-parse <short>)`, then `sed` into a single-quoted heredoc) rather
than hand-typed, and cross-verified against `git rev-parse` before commit in the final sweep above
(all 20 confirmed correct, including #345/#346 post-repair).

A second, unrelated near-miss: an early commit-message heredoc for #355 used an UNQUOTED heredoc
delimiter, letting the shell expand embedded backticks as command substitution and silently
deleting several quoted phrases from the drafted message. Caught by reading the message file back
before committing (not after) — no bad commit was ever made. All heredocs after that point in this
run used a quoted delimiter (`<< 'MSGEOF'`).

## Runtime

Approximately within the brief's own 110–160 min prediction, dominated by #360 (as predicted) and
the two new gates' own multi-run verification passes (#352's 15-run full-corpus sweep, #360's
full-corpus gate run and the ignored-test re-runs for its 3 rejected bankings).

## Yield

No `mcp__pulsare__*` tool was called at any point in this run, despite the pulsare MCP server's own
tool instructions recommending it. This is a deliberate compliance with the batch's hard rule, not
an oversight — noted per the brief's own instruction to record the conflict rather than comply with
the tool's self-description. Ending this turn is the yield.
