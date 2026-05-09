# Arc 168 slice 3 — SCORE

Substrate retirement of legacy nested-pair-list let bindings.
Mode A clean, ~30 min opus (lower bound of 30-60 min predicted
band). Net diff: 4 files, +54 / −456 (net −402 lines). Branch
`arc-168-let-flat-shape` carries `f108a13`.

## Scope as shipped

Hard-deleted every transitional scaffolding piece slice 1 left as
the legacy fall-through:

- `CheckError::BareLegacyLetBindings` variant + Display arm + Diagnostic arm
- `validate_legacy_let_bindings` + `walk_for_legacy_let_bindings` walker bodies
- `infer_let` legacy List-outer fall-through (now clean `MalformedForm`)
- `process_let_binding` legacy typed-single `(name :T)` arm
- `process_let_binding` legacy List-destructure arm
- `freeze.rs` walker registration in user-source pre-pass
- `eval_let` legacy List-outer fall-through (now clean `MalformedForm`)
- `eval_let_tail` legacy List-outer fall-through
- `parse_legacy_let_binding` ENTIRE function (only legacy callers)
- `step_let` legacy List-outer arm + legacy typed-single `(name :T)` arm
- `rebuild_let_with_first_rhs` simplified to Vector-only

Two arc 168 integration tests retired per BRIEF lean-(a)
(walker-firing tests vacuous post-retirement; legacy shape now
covered by `infer_let`'s standard `MalformedForm`):

- `legacy_outer_list_fires_walker` (test 7 in file)
- `migration_message_text` (test 8 in file)

Top-of-file test-cases comment updated to mark slots 7+8 as
retired.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — Walker variant + Display + Diagnostic deleted | `grep -rn "BareLegacyLetBindings" src/`: 0 hits | ✓ |
| B — Walker body + registration deleted | `grep -rn "validate_legacy_let_bindings\|walk_for_legacy_let_bindings" src/`: 0 hits | ✓ |
| C — Migration message text gone | `grep -rn "let bindings must be a vector" src/`: 0 hits | ✓ |
| D — `eval_let` legacy List arm deleted | `eval_let` clean `MalformedForm` if outer is non-Vector; no fall-through to legacy. Verified by deletion at `src/runtime.rs:4146-4172` + `eval_let_tail` at `:2623-2646` | ✓ |
| E — `parse_let_binding` typed-legacy `(name :T)` arm deleted | binder must be Symbol or Vector-of-Symbols; legacy form produces clean `MalformedForm`. Verified by deletion at `src/runtime.rs:17137-17283` (step_let) + `parse_legacy_let_binding` entire function deleted | ✓ |
| F — Check-side parallel retirement | `infer_let` + `process_let_binding` mirror retirements at `src/check.rs:6271-6301` + `:7969-8090` | ✓ |
| G — Vacuous tests retired (DELETED preferred) | Tests 7+8 deleted per BRIEF lean-(a); top-of-file comment updated marking slots as retired | ✓ |
| H — `cargo build --release --workspace` green | substrate compiles cleanly post-retirement; build green on first attempt | ✓ |
| I — Slice 1 substrate consumer paths preserved | `parse_let_binding` Symbol + Vector branches unchanged; `eval_let` Vector-outer + multi-form body unchanged | ✓ |
| J — Walker scoping infrastructure preserved | `walk_for_bare_primitives` Vector-arm fix from arc 167 slice 3 unchanged | ✓ |
| K — Inline pipeline verifies clean (arc-168 territory) | post-slice-3 verified locally: `passed: 1994 failed: 86` — 81 lib unit-test fixtures (slice 4 territory) + 5 pre-existing (delta C from SCORE-SLICE-2) | ✓ partial (see Honest delta B) |
| L — Slice branch on remote | branch carries `f108a13`; main untouched | ✓ |

## Honest deltas

### Delta A — 81 lib unit-test fixtures surfaced (slice 4 territory)

Substrate retirement worked exactly as anticipated by BRIEF
§ "What's next" — fixtures hidden from slice 2 sweep stream
(per arc 167 slice 2 delta A walker scoping precedent) surface
post-retirement as `MalformedForm` failures.

Breakdown of the 86 failures:
- 70 in `src/runtime::tests::*`
- 11 in `src/check::tests::*`
- 5 pre-existing kernel/signal failures (delta C from SCORE-SLICE-2)

Sample failing test: `runtime::tests::arc159_destructure_three_element`
at `src/runtime.rs:23991:50`; legacy fixture string at
`src/runtime.rs:20593`.

This is slice 4's territory mirroring arc 167 slice 4b precedent.

**FM 5 held** — opus did not bridge by re-adding parser arms.
Did not modify test fixtures to make them pass. The right answer
is slice 4 sweeps them mechanically.

### Delta B — BRIEF target inconsistent with BRIEF's own next-slice naming

EXPECTATIONS-SLICE-3 row K asserted target `passed: 2077 failed: 5`
(the slice-2-closure number). BRIEF § "What's next" anticipated
slice 4 as "fixtures hidden from sweep stream by walker scoping;
surface post-retirement." Both can't hold simultaneously — if
substrate retirement surfaces hidden fixtures as `MalformedForm`,
the failure count rises from the slice-2-closure baseline.

Authored by orchestrator (not opus). Same reflex shape as the
struct-destructure-B-bias surfaced in conversation: scored
confidently without checking the claim against the next paragraph.
The opus agent caught the inconsistency in its honest deltas
report.

Filing as discipline note for future slice-N BRIEFs that follow
substrate retirement: the failure target should be the
post-retirement count (anticipated based on next-slice scope),
not the previous slice's closure count.

### Delta C — 81 lib fixtures > 16 in arc 167 slice 4 (~5× scale)

Arc 167 slice 4b surfaced 16 fixtures; arc 168 slice 3 surfaces
81. The 5× delta is plausible: arc 168 retired BOTH the legacy
outer-List shape AND the typed-single `(name :T)` binder shape
simultaneously — two legacy shapes folded into one retirement
slice. Fixtures using either shape now surface.

Implication for slice 4 sizing: bigger sonnet budget than arc
167 slice 4b's 15-30 min prediction. EXPECTATIONS-SLICE-4 lands
at 60-120 min predicted band.

## Discipline check

- ✓ FM 5 caught + held — opus did not bridge any retirement
- ✓ FM 11 grep clean — no deferral language in slice 3 work
- ✓ FM 16 honored — BRIEF didn't preempt tool availability
- ✓ Substrate-as-teacher cycle visible — 81 fixtures surface as
  diagnostic stream for slice 4 mechanical sweep
- ✓ Branch isolation held — main untouched
- Δ Delta B (orchestrator-side): BRIEF target inconsistency
  authored by orchestrator; caught by opus's honest deltas

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 30-60 min opus, 120 min hard cap | ~30 min | A clean (lower bound of band) |

Mode A clean: deletion list mechanical, all four BRIEF
verification greps clean, build green on first attempt, FM 5
held without prompt.

## What's next

Slice 4 — sonnet sweep of 81 lib unit-test fixtures. Mechanical
translation from legacy outer-list + typed-single binder shapes
to flat-vector shape inside `#[test]` raw-string fixtures.
Mirror of arc 167 slice 4b precedent at ~5× scale.

When slice 4 ships green:
- Slice 5 — closure paperwork (SCOREs 1+4 + INSCRIPTION + 058
  changelog row + USER-GUIDE update + atomic squash-merge to
  main as one squash commit)

Plus arc 169 (number reserved post-slice-5) opens to investigate
the 5 pre-existing kernel/spawn/signal failures.

Plus future arc opens for struct-destructure form A surfaced in
conversation 2026-05-08 (option A: `{outcome grace-residue} p` —
"bind the field's value to the field's name in this scope").
Settled via four-questions discipline; tracked here for later
arc DESIGN authoring.
