# SCORE 7g — replay batch 4g, grok-rete #261 → #280

Batch complete: 20 REPLAY commits, tip `398cc3321`. Every row below answers with the command run
and its actual output. One record repair was found and folded before this SCORE was written (see
E20).

## E1 — 20 steps, correctly subjected

Command: `scripts/replay/verify-step-record.sh 7b58b6cbd HEAD 261 280`

```
step-range: #261..#280 each present exactly once, sources match
step-record: complete
```

exit 0 (also re-run under `GIT_NO_REPLACE_OBJECTS=1`, identical output). **PASS.**

## E2 — docs-only steps are docs-only

Command, per the 15 docs-only steps:

```
for n in 261 263 264 265 267 268 269 271 272 273 275 276 277 279 280; do
  sha=$(git log --format='%H %s' 7b58b6cbd..HEAD | grep "#${n})" | awk '{print $1}')
  git show --name-only --format= "$sha" | grep -vE '^docs/|\.md$'
done
```

No output — every file in every one of the 15 commits is under `docs/`. **PASS.**

## E3 — the #274 meta-gate is GREEN at HEAD, both arms

Command: `cargo nextest run --release -E 'test(every_discovering_gate_declares_how_it_knows_it_reached_something)'`

```
Starting 1 test across 46 binaries (5680 tests skipped)
    PASS [   0.004s] (1/1) wat::lint every_walking_gate_declares_non_vacuity::every_discovering_gate_declares_how_it_knows_it_reached_something
Summary [   0.011s] 1 test run: 1 passed, 5680 skipped
```

1 test run, PASS — neither `undeclared` nor `hollow` fires. **PASS.**

## E4 — all ten repair targets carry a declaration

Command:

```
grep -LiE 'non-vacuity|rune:lint\(vacuity-guard\)' \
  tests/lint/every_tracked_wat_parses.rs tests/lint/holon_is_vsa_only.rs \
  tests/lint/ignore_reason_justified.rs tests/lint/nested_program_starts.rs \
  tests/lint/no_bare_is_err.rs tests/lint/no_bootstrap_path_in_committed_rust.rs \
  tests/lint/no_error_flattening_helper.rs tests/lint/one_variant_separator.rs \
  tests/lint/tracked_wat_dir_is_stdlib_sources.rs tests/lint/every_ungated_wat_checks.rs
```

Empty output — no file lacks a declaration. **PASS.**

## E5 — the declarations are not HOLLOW

Re-read each of the ten by hand at commit time (logged per-file in REPLAY-LOG.md's #274 section):
five already had a real guard and needed only the `// NON-VACUITY:` marker line placed directly
above it (`every_tracked_wat_parses.rs:paths.len() > 1000`, `holon_is_vsa_only.rs:files.len() >
50`, `nested_program_starts.rs:total >= 141`, `tracked_wat_dir_is_stdlib_sources.rs:!tracked
.is_empty()`/`!loaded.is_empty()`, `ignore_reason_justified.rs:total_real_ignores >=
FROZEN_ALLOWLIST.len()`); the hollow one (`every_ungated_wat_checks.rs`) got a fresh `//
NON-VACUITY:` line directly above its existing `:46` guard, prose left untouched; four had no
guard and one was authored each with a floor measured on this tree (`no_bare_is_err.rs`,
`no_bootstrap_path_in_committed_rust.rs`, `no_error_flattening_helper.rs`,
`one_variant_separator.rs`). The meta-gate's own `hollow` arm (which specifically detects a marker
with no assertion under it) passed at E3 above — a positive proof, not merely an absence of
complaint, since this exact arm fired once, correctly, before the `every_ungated_wat_checks.rs`
repair. **PASS.**

## E6 — #278's dead path was NOT recreated

Command: `ls wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`

```
ls: cannot access 'wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat': No such file or directory
```

Absent (exit 2). **PASS.**

## E7 — no INERT rune was added to the moved copy

Command:
`grep -c 'rune:lint(rete-name-unminted)' tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat`

```
0
```

**PASS.**

## E8 — the #278 gate is green

Command: `cargo nextest run --release -E 'test(every_rete_name_in_wat_scripts_code_resolves)'`

```
Starting 1 test across 46 binaries (5680 tests skipped)
    PASS [   0.098s] (1/1) wat::lint rete_names_in_wat_scripts_resolve::every_rete_name_in_wat_scripts_code_resolves
Summary [   0.106s] 1 test run: 1 passed, 5680 skipped
```

Plus its three sibling controls: `cargo nextest run --release -E 'test(known_forms_are_real) +
test(prose_in_rust_does_not_attest_a_name) + test(prose_control_holds)'`

```
Starting 3 tests across 46 binaries (5678 tests skipped)
    PASS [   0.037s] (1/3) wat::lint rete_names_in_wat_scripts_resolve::prose_control_holds
    PASS [   0.048s] (2/3) wat::lint rete_names_in_wat_scripts_resolve::known_forms_are_real
    PASS [   0.050s] (3/3) wat::lint rete_names_in_wat_scripts_resolve::prose_in_rust_does_not_attest_a_name
Summary [   0.057s] 3 tests run: 3 passed, 5678 skipped
```

All green, N > 0 selected in both runs. **PASS.**

## E9 — `wat-scripts/fixes/` edited at #278 and nowhere else

Command: `git log --format='%h %s' 7b58b6cbd..HEAD -- wat-scripts/fixes/`

```
9494092b7 REPLAY(grok-rete #278): lint: every rete name in wat-scripts CODE resolves — prose may name a retired form
```

Only #278's commit. **PASS.**

## E10 — every produced `.wat` checks

`--check` on #262's 6 and #278's remaining 4 (5 minus the dropped dead-path file, per E6):

```
#262:
tests/rete/probe_arc278_field_span_nested.wat                              rc=0
tests/rete/probe_arc278_nested_wall_arity.wat                              rc=1  (deliberate refusal — RhsArityMismatch)
tests/rete/probe_arc278_nested_wall_missing_fields.wat                     rc=1  (deliberate refusal — RhsMissingFields)
tests/rete/probe_arc278_nested_wall_ok.wat                                 rc=0  (control; fires, derives 1 fact)
tests/rete/probe_arc278_nested_wall_positional_retired.wat                 rc=1  (deliberate refusal — RhsPositionalConstructionRetired)
tests/rete/probe_arc278_nested_wall_unknown_field.wat                      rc=1  (deliberate refusal — UnknownField)

#278:
wat-scripts/fixes/rete-oracle-sigil.wat                                    rc=0
wat-scripts/fixes/rete-where-per-type-spelling.wat                         rc=0
wat-scripts/fixes/type-query-to-defquery.wat                               rc=0
wat-scripts/scratch-pad/probe-arc278-57-round1b-parametric-and-hof.wat     rc=0
```

The four `rc=1` fixtures are #262's own deliberately-refusing probes (`arity`/`missing_fields`/
`positional_retired`/`unknown_field`), each named test asserting `!ok` — reason stated, per E10's
"or a deliberate `.bad`, with its reason" clause. **PASS.**

## E11 — named tests at the code steps

Per-step counts (all captured live at commit time, re-confirmed at HEAD):

- #262: 16/16 (`probe_arc278_field_span`'s 5 + `probe_arc278_nested_wall`'s 5 + `probe_arc278_
  enum_variant_typo`'s 6)
- #266: 10/10 (`no_ceiling_raise_in_rete` + `probe_arc278_fixpoint_round_cap`'s 9)
- #270: 3/3 (`the_broken_doc_link_ledger_has_no_duplicate_keys`,
  `the_unresolved_link_extractor_still_matches_rustdocs_format`,
  `no_broken_intra_doc_link_outside_the_frozen_ledger`)
- #274: 42/42 (meta-gate's 15 + 27 across the ten repaired gates)
- #278: 24/24 (`rete_names_in_wat_scripts_resolve`'s 20 + `no_ceiling_raise_in_rete`) plus
  `every_wat_scripts_file_loads_on_the_current_runtime` (1/1, 169s)

All green, N > 0 in every `-E` selection. **PASS.**

## E12 — #274's repair landed AT #274, not folded backward

Command: `git show --name-only 8322fe021 | grep tests/lint | wc -l`

```
30
```

The #274 REPLAY commit itself carries all 30 `tests/lint/*.rs` files — grok's 20 plus the 10
repairs — not a later step. **PASS.**

## E13 — finding 33's class was actively looked for

Each of the five code steps' commit body carries an explicit "finding 33 grepped: YES" line
(`git show -s --format=%B <sha> | grep -i 'finding 33'` on #262/#266/#270/#274/#278, all match).
#274 and the 15 docs-only steps state "not applicable" where relevant (no rename in scope) or
carry no code diff at all. Silence nowhere; every code step answers explicitly. **PASS.**

## E14 — the checkpoint

Per BRIEF-7g's own anchor list, `scripts/floor.sh` and `cargo clippy --release --all-targets` are
explicitly forbidden to the executor ("Do NOT run scripts/floor.sh, cargo clippy, or run5 — the
orchestrator weighs those centrally and uncontended; a gate run while you work in the tree is a
FALSE result"). **Not checked, because the brief forbids running them from inside this session —
they are the orchestrator's own row to run centrally.**

## E15 — test-count delta ACCOUNTED FOR

Same reason as E14 — this row depends on a floor run this executor is forbidden to make. **Not
checked, because the floor itself was not run.** For the record, the delta this batch's own diff
predicts is entirely additive: +15 unit tests in `every_walking_gate_declares_non_vacuity.rs`
(#274) + 5 new tests in `probe_arc278_nested_wall.rs` (#262) + 3 new tests in
`no_new_broken_doc_link.rs` (#270), no test deleted or renamed anywhere in the batch (one test in
`probe_arc278_field_span.rs` was renamed at #262 —
`nested_constructor_field_is_never_validated_at_all` →
`a_nested_constructor_names_the_field_keyword_not_the_whole_form` — net count unchanged).

## E16 — the checkpoint (spot re-run of the walls)

Full wall re-run at HEAD after landing #280 and after the E20 record repair (tree unchanged by the
repair, see E20):

```
lint-subset:          192 passed  (identical to #278's recorded 192)
kind(lib):           1490 passed  (identical to #278's recorded 1490)
doctest:                8 passed  (identical to #278's recorded 8)
census:               2122 files, no STOP-8  (identical to #278's recorded 2122)
nested-program-gate:  3/3, 5678 skipped  (identical to #278's recorded 3/3)
```

Identical to the last code step's (#278) own recorded numbers. **PASS.**

## E17 — no knowingly-red commit

No repair commit was appended after #280 — the one record repair found (E20) was folded into the
existing #262/#266/#270/#278 commits themselves via `git filter-branch --msg-filter` (message-only,
tree hash unchanged, verified twice by `git rev-parse HEAD^{tree}` before/after). No REPLAY commit
was ever committed in a red state; every commit's gates were run and confirmed green before that
commit was made. **PASS.**

## E18 — every artifact a body names exists

Every `.census/…txt` cited across the five code steps' commit bodies:

```
.census/2026-09-16T10-00-08Z.txt   OK   (#262)
.census/2026-09-16T10-11-13Z.txt   OK   (#266)
.census/2026-09-16T10-23-28Z.txt   OK   (#270)
.census/2026-09-16T10-35-08Z.txt   OK   (#274, cited as the prior-baseline for #278)
.census/2026-09-16T10-57-22Z.txt   OK   (#278)
.census/2026-09-16T09-04-34Z.txt   OK   (#260's, cited as #262's prior-baseline)
```

All present on disk (gitignored, local-only artifacts as designed). **PASS.**

## E19 — every repair visible to `push`

Command: `git replace -l`

```
(empty)
```

Gate re-run under `GIT_NO_REPLACE_OBJECTS=1`:

```
step-range: #261..#280 each present exactly once, sources match
step-record: complete
```

exit 0 either way — **0 replace refs**. The one repair made (E20) was a `git filter-branch
--msg-filter` rewrite of the actual commit objects themselves (not an overlay), so it is visible
to `push` by construction; there is no `refs/replace/` entry to hide behind. **PASS.**

## E20 — no verdict line is WRAPPED

This is the row this batch's own self-review caught red, pre-yield. First-draft bodies for
#262/#266/#270/#278 wrote `census: <file> files=<N>; --diff (vs #M's <file2>) no STOP-8` — a
parenthetical sitting between `--diff` and `no STOP-8` that the record gate's `census: .*--diff no
STOP-8` pattern requires to be contiguous. `verify-step-record.sh` reported all four `MISSING`
before this SCORE was drafted. Repaired via `git filter-branch --msg-filter` (two passes: first
joining a two-line wrap, second moving the `(vs #M's …)` clause to AFTER `no STOP-8`) — tree hash
`20201ac368fccab248fd691eaa7b17953b6e6d7d` confirmed identical before and after both passes.
Re-verification (E1, E19) both now pass. Confirmed every required verdict line across the batch is
a single-line match:

```
for h in $(git log --reverse --format=%H 7b58b6cbd..HEAD); do
  # (per-step required patterns checked individually against %B, grep -c each, all == 1)
done
```

— every one of #262/#266/#270/#274/#278's required lines (`census: .*--diff no STOP-8`,
`nested-program-gate: PASS`, `lint-subset: [0-9]+ passed`, `kind\(lib\): [0-9]+ passed`,
`doctest: [0-9]+ passed`) matches exactly once, on one line. **PASS (after the fold; RED found
and repaired before yielding).**

## What would have rejected this batch — none of it happened

The meta-gate was never weakened, allowlisted, or deferred (#274 landed red on arrival, exactly as
predicted, and was repaired at the step). No `// NON-VACUITY:` marker was left with no assertion
beneath it (E5). The #278 probe was never recreated at its dead path (E6), and no inert rune was
added under `tests/resolve/` (E7). No `refs/replace/` entry exists (E19). No `.wat` was left
failing `--check` without a stated, checkable reason (E10). The batch's one RED — the record-line
wrap at E20 — was caught by self-review before yield, evidenced verbatim, and repaired by rewriting
the actual commit objects rather than by any overlay or after-the-fact commit.
