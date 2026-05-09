# Arc 168 slice 4 — SCORE

Sonnet swept 81 legacy let-binding fixtures in `src/runtime.rs` (70)
+ `src/check.rs` (11) lib unit-test embedded wat strings. ~35 min
runtime, single mechanical commit. Workspace landed at 2074/6
post-sweep; closure investigation surfaced two parallel substrate
gaps slice 1 left in the deadlock walkers, both fixed in
follow-up commits. Final state: 2075/5 → 2080/0 after slice 4 follow-up D (the 5 were sweep misses, not pre-existing
kernel/spawn/signal failures unrelated to arc 168).

## Scope as shipped

### Sonnet's slice 4 commit (`3b34d62`)

71 + 11 = 82 fixtures translated:
- `src/runtime.rs`: 70 anticipated + 1 proactive migration of
  `let_destructure_arity_mismatch_errors` (sonnet delta B)
- `src/check.rs`: 11 fixtures matching slice 3's surface

Three legacy shapes covered per BRIEF recipe:
- Bare-symbol nested-pair-list: `((name expr) ...)` → `[name expr ...]`
- Typed-single legacy: `(((name :T) expr))` → `[name expr]`
- List-destructure: `(((a b c) rhs))` → `[[a b c] rhs]`

Empty-bindings `()` → `[]` covered.

### Slice 4 follow-up A (`89fd0ac`) — substrate Δ-A

`check_let_for_scope_deadlock_inferred::binding_names` had Vector
arm gap. Symbol + List(parts) were matched; Vector(symbols) — the
arc 168 flat-shape destructure binder — fell through to vec![],
so destructure-introduced names never reached the ScopeDeadlock
collector. Fix: 5-line addition of the Vector arm.

### Slice 4 follow-up D (`bd39282`) — pre-existing-classification correction

User direction 2026-05-09 during closure review: *"there should be
zero kernel/signal failures - let's pivot and address those before
we merge anything to main."*

Investigation showed all 5 "pre-existing" failures originally
classified by sonnet's slice 4 report were actually arc 168 sweep
misses. The orchestrator propagated the wrong framing across SCORE-2
delta C, SCORE-3 honest delta C, SCORE-4 calibration, INSCRIPTION
"Workspace state" section, AND the 058 changelog row — five
artifacts carrying the same unverified claim.

**Root cause per failure:**
1. `fork_program_round_trip_via_pipes` — slice 2 sweep dropped one
   closing `)` from inner-src string while migrating outer
   `((... ))` → `[...]`. Inner wat program unbalanced; child
   parse-failed; parent read None.
2-5. `presence_proof_hello_world`, `programs_are_atoms_hello_world`,
   `sigterm_to_cli_cascades_via_polling_contract`, `sigterm_cascades_two_levels_via_process_group`
   — wat fixtures inside `const FOO: &str = r#"..."#` raw-string
   declarations entirely missed by slice 2 sweep (sweep targeted
   test bodies, not const declarations). All used legacy
   `((name expr) ...)` outer-list shape.

**Discipline failure pattern recorded in memory
`feedback_pre_existing_verification.md`:** sonnet's "stash
round-trip showed pre-existing" is a tautology when the current
branch retired the syntactic path the test depended on. The 5-second
verification: grep the failing test's fixture for the legacy syntax
pattern; if hits, sweep miss not pre-existing.

**Fixes shipped in `bd39282`:**
- 1 paren restoration in `tests/wat_arc104_fork_program.rs`
- 4 fixture migrations in `crates/wat-cli/tests/wat_cli.rs`
- 2 of the 4 const program migrations needed an extra trailing `)` after
  migration (the legacy structure had an inner pair close paren the
  migration dropped on top of the outer bindings list close)

**Final state:** `passed: 2080 failed: 0`. Workspace clean.

### Slice 4 follow-up B+C (`751addb`) — substrate Δ-B + lib fixture sweep

Two more parallel substrate fixes + 6 lib unit-test fixture
migrations covering "accidentally green" tests:

**Δ-B substrate fixes:**
- `walk_for_pair_deadlock` at `src/check.rs:3032` — early-returned
  on Vector outer; walker never ran on flat-shape lets. Fix:
  desugar Vector outer into `Vec<List([binder, rhs])>` mirroring
  `bindings_pairs` in `infer_let`.
- `extend_pair_scope_with_tuple_destructure` at `src/check.rs:3251`
  — destructure binder match accepted only List, not Vector.
  Fix: accept both shapes for the binder.

**Δ-C lib fixture sweep:**
6 fixtures previously "accidentally green" via MalformedForm-
satisfies-assertion now exercise the actual rule paths:
- `arc_128_inner_scope_deadlock_skipped_in_sandboxed_forms`
- `arc_131_handlepool_without_sender_silent`
- `arc_133_tuple_destructure_silent_when_clean`
- `arc_133_tuple_destructure_pair_check_fires`
- `arc_134_thread_input_output_does_not_fire`
- `arc_134_no_recv_in_fn_body_does_not_fire`

Migration applied to embedded wat: legacy outer → flat-vector;
embedded fn-sigs migrated where present; legacy list-destructure
binders migrated to vector-destructure.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `cargo test` count drops to 0 failed | post-follow-up-D: `passed: 2080 failed: 0` (initially landed at 2075/5; user-directed pivot revealed those 5 were arc 168 sweep misses, fixed in follow-up D) | ✓ |
| B — `src/runtime.rs` swept | 70 + 1 fixtures migrated; remaining greps clean | ✓ |
| C — `src/check.rs` swept | 11 (slice 4) + 6 (Δ-C follow-up) = 17 fixtures migrated | ✓ |
| D — Substrate untouched (slice-4 sonnet commit) | sonnet's `3b34d62` touches only `#[test]` raw-string fixtures; substrate diff = 0 | ✓ |
| E — Tests/assertions untouched (slice-4 sonnet commit) | NO assertion edits in sonnet's commit | ✓ |
| F — Mechanical translation only | each migration is binder-shape change + fn-sig where present; no semantic reshapes | ✓ |
| G — Slice branch on remote | branch carries 4 commits since slice 3; main untouched | ✓ |
| H — Inline pipeline verifies clean | sonnet's report references the inline pipeline; FM 9 verified locally | ✓ |
| I — All three legacy shapes covered | bare-symbol nested-pair-list; typed-single `(name :T)`; list-destructure `((a b c) rhs)`; empty `()` | ✓ |
| J — `wat/core.wat` defn macro untouched | `git diff wat/core.wat`: no changes | ✓ |
| K — FM 5 held throughout | sonnet honored on the originally-failing test (Δ-A); orchestrator honored when Δ-C migration revealed Δ-B (STOP, fix at substrate, not bridge) | ✓ |

## Honest deltas

### Delta A — sonnet stopped on `arc_133_tuple_destructure_with_handlepool_fires` (substrate gap)

Pre-sweep this test was failing with `MalformedForm` from the
retired outer-list parser. After mechanical Shape 3 migration
`(((pool driver) (:my::spawn-svc)))` → `[[pool driver] (:my::spawn-svc)]`,
the form parsed correctly but `check_let_for_scope_deadlock_inferred`
no longer fired `ScopeDeadlock`. The check infrastructure for the
arc 133 ScopeDeadlock rule was built against the legacy
`process_let_binding` destructure path; the flat `[A B]` binder
shape didn't populate `extended` with the same type information
that triggers the rule.

Sonnet honored FM 5 (STOP, no bridge). Delta A surfaced as the
substrate gap.

Orchestrator-side fix (commit `89fd0ac`): added Vector arm to
`binding_names` extraction. 5-line change.

### Delta B — Sonnet proactive: `let_destructure_arity_mismatch_errors` accidentally green

Test fixture used `(((a b c) p))` shape and was NOT in the
failing list (it expected `MalformedForm` which the retired
outer-list parser delivered accidentally). Sonnet migrated to
`[[a b c] p]` for correctness. Test remains green (proper arity-
mismatch error from the new substrate path).

This is the +1 / -1 calibration drift: BRIEF predicted 70
runtime sites; actual 71. Reasonable proactive call.

### Delta C — 6 check.rs tests in legacy shape (orchestrator follow-up)

Sonnet stayed in BRIEF scope (81 fixtures); flagged 6 additional
check.rs tests as out-of-scope-but-needs-cleanup. These had no-
fire / expect_err assertions accidentally satisfied by
MalformedForm from retired parser; rule body never executed.

Migration revealed Delta B substrate gap when
`arc_133_tuple_destructure_pair_check_fires` flipped from
"accidentally green" to "actually-failing because pair-deadlock
walker doesn't traverse flat-shape lets."

Pattern: same family as Delta A. Slice 1's substrate retirement
created parallel surfaces in BOTH deadlock walkers (ScopeDeadlock
+ ChannelPairDeadlock); slice 4 sweep + Delta C surfaced both.

Both fixes shipped cleanly in commit `751addb`. FM 5 held
throughout — the migration revealing real bugs surfaced as
honest delta, not bridged.

### Delta D — Sonnet calibration

| Predicted | Actual | Mode |
|-----------|--------|------|
| 60-120 min sonnet | ~35 min | A clean (lower bound) |

Tool-call ratio (158 tool uses for 82 sites) ≈ 2 calls/site —
matches slice 2 calibration. The brief Edit-tool-per-fixture
approach scaled cleanly at this size. For larger sweeps (~563
sites in slice 2), the python-script approach noted in
SCORE-SLICE-2 delta D remains the cost-aware path.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| Slice 4 sweep ~81 sites, 60-120 min sonnet | ~35 min, 158 tool uses, 2074/6 (post-sweep) → 2075/5 (post-follow-ups A+B+C) → 2080/0 (post-follow-up D after user-directed pre-existing-classification correction) | A clean per slice; orchestrator-side discipline correction caught at closure review |

## Discipline check

- ✓ FM 5 caught + held three times: sonnet on Δ-A; orchestrator
  on Δ-B (Δ-C migration revealing the parallel walker gap)
- ✓ FM 9 honored — re-ran cargo test locally between each
  follow-up commit; counts verified before scoring
- ✓ FM 11 grep clean — no deferral language in slice 4 or
  follow-up commits
- ✓ FM 16 honored — BRIEF didn't preempt tool availability
- ✓ Substrate-as-teacher cycle complete — substrate retirement
  created walker gaps that surface via test failures; each gap
  fixed at substrate level, not bridged
- ✓ Branch isolation held — main untouched throughout

## What's next

Slice 5 — closure paperwork. Includes:
- SCORE-SLICE-1 (slice 1's ship was never explicitly scored;
  closure convention)
- INSCRIPTION.md (arc 168 closure)
- 058 changelog row (FOUNDATION-CHANGELOG.md in trading lab)
- USER-GUIDE update (let flat-shape + multi-form body)
- Atomic squash-merge to main as one squash commit

Plus arc 169 (struct-destructure form A) — DESIGN.md drafted
2026-05-08 in this slice's branch; v1-closure blocker for arc 109.

~~Plus future arc opens for the 5 pre-existing kernel/spawn/signal~~ [STRUCK — those 5 were arc 168 sweep misses, fixed in follow-up D `bd39282`. The "pre-existing" framing was wrong. No follow-up arc needed for kernel/signal
failures (number reserved post-arc-168 closure).
