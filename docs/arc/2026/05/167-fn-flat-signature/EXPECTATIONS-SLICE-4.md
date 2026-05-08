# Arc 167 slice 4 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 30-60 minutes (opus agent).**

Reasoning: substrate-judgment-medium work — coordinated multi-file
deletions across `src/check.rs`, `src/runtime.rs`, `src/freeze.rs`,
plus 2 test cases retired. Code-deletion arcs ship faster than
code-addition arcs because the verification is just "does it still
compile + tests pass." Comparable to arc 154 slice 2 walker
retirement (~30 min) or arc 162 lambda internal-rename (~60 min for
~353 sites; this is much smaller).

**Time-box (2× upper-bound): 120 minutes.** If opus is still
iterating at 60 min, in-flight check; hard cap at 120 via TaskStop
+ Mode B-time-violation.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | `BareLegacyFnSignature` variant deleted from `CheckError` | `grep -n "BareLegacyFnSignature" src/check.rs` returns 0 hits |
| B   | Display impl arm deleted | git diff confirms removal |
| C   | Diagnostic impl arm deleted | git diff confirms removal |
| D   | `walk_for_legacy_fn_signature` body deleted | `grep -n "walk_for_legacy_fn_signature" src/check.rs` returns 0 hits |
| E   | `validate_legacy_fn_signature` registration removed from `freeze.rs:599-616` | git diff confirms removal in freeze.rs |
| F   | Migration-hint string constants deleted | `grep -n "fn signature must be a vector binding form" src/` returns 0 hits |
| G   | `parse_legacy_fn_signature` (runtime) deleted | `grep -n "parse_legacy_fn_signature" src/runtime.rs` returns 0 hits |
| H   | `parse_legacy_fn_signature_for_check` deleted | `grep -n "parse_legacy_fn_signature_for_check" src/check.rs` returns 0 hits |
| I   | `eval_fn` 2-arg legacy arm removed; only 4-arg canonical path remains | git diff confirms removal of the legacy arm |
| J   | Tests 5 + 6 retired (deleted or replaced cleanly) | `grep -n "legacy_nested_sig_fn_fires_walker\|legacy_nested_sig_defn_fires_walker_via_macro" tests/wat_arc167_fn_flat_signature.rs` returns 0 hits if deleted, OR returns hits with new MalformedForm assertions if replaced |
| K   | `cargo build --release --workspace` green | substrate compiles cleanly post-deletion |
| L   | `./scripts/cargo-test-summary.sh` shows 0 failed | workspace stays green; total may be 2067 (=2069-2) if tests 5+6 deleted, or 2069 if replaced |
| M   | `walk_for_bare_primitives` Vector arm preserved at `src/check.rs:2200+` | git diff confirms NO modification to that arm; permanent infra stays |
| N   | `wat/core.wat` defn macro unchanged | git diff confirms no edit to the canonical macro |
| O   | `parse_fn_signature` + `parse_fn_signature_for_check` (canonical paths) unchanged | git diff confirms no edit to the canonical parsers |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Unexpected references to the legacy parser.** If `grep
  parse_legacy_fn_signature src/ tests/` reveals references in
  files you haven't yet edited, STOP and report — there's a
  caller we didn't catalog.
- **Test 9 `reflection_on_flat_defn_resolves` fails post-deletion.**
  If retirement breaks reflection in unexpected ways, that
  signals a hidden dependency on the legacy arm; STOP and report
  rather than bridging.
- **Dual-arm branching deeper than expected.** Some calling
  contexts may have shape-dispatched (`if args.len() == 4 ... else
  ... legacy ...`). If a single deletion surfaces a complex
  branching pattern we didn't catalog, STOP and report.
- **`Diagnostic` impl arm retirement.** Some `CheckError` variants
  carry `Diagnostic` impls separate from `Display`. If retirement
  of `BareLegacyFnSignature` requires also removing a Diagnostic
  arm, do so — but if there's any non-trivial logic (like span
  helpers) entwined that you'd need to disentangle, surface it.
- **stdlib usage of legacy syntax.** Slice 3 swept `wat/*.wat` to
  the new shape. If you find stdlib still using legacy syntax
  somewhere we missed, STOP and report — that's a slice 3 leftover,
  not a slice 4 substrate retirement.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial / Mode C
failed). Compare to predicted 30-60 min band.

## What's next (orchestrator-side, post-slice-4)

When slice 4 ships green, slice 5 closure paperwork:
- SCORE-SLICE-3.md (already deferred)
- SCORE-SLICE-4.md (this slice)
- INSCRIPTION.md (full arc 167 closure)
- 058 changelog row (one row covering slices 1-4)
- USER-GUIDE update (defn + fn sections show flat shape; legacy
  examples removed)
- Atomic squash-merge slice branch to main
- Branch can be deleted after merge (audit trail in remote stays)

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-4.md
to the slice branch after scoring all rows + reviewing the diff.
