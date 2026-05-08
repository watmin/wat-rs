# Arc 167 slice 3 — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-120 minutes (sonnet agent).**

Reasoning: substrate-as-teacher mechanical sweep across ~150-200
sites total. Per slice 2 SCORE calibration band (50-200 failures
→ ~60-120 min sonnet) + ~30 stdlib grep sites. Each migration is
the same mechanical translation per the walker's verbose message.
Comparable in shape to arc 159 slice 2 (~951 sites for let
untyped); fn-sig sweep should be smaller because fn is less
common than let in user code.

**Time-box (2× upper-bound): 240 minutes.** If sonnet is still
iterating at 120 min, in-flight check; hard cap at 240.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A   | Phase 1 — test-driven sweep complete | `cargo test --release --workspace --no-fail-fast` returns 0 failed (down from 152) |
| B   | Phase 2 — stdlib grep sweep complete | `grep -rn -E '\(:wat::core::(fn\|defn) +\('` returns 0 hits in `wat/` + `wat-tests/` + `crates/` (excluding intentional walker-firing tests in `tests/wat_arc167_fn_flat_signature.rs` cases 5/6) |
| C   | New flat-shape fn/defn callsites compile cleanly | `cargo build --release --workspace` green |
| D   | Walker still fires for genuine legacy callsites | tests `legacy_nested_sig_fn_fires_walker` + `legacy_nested_sig_defn_fires_walker_via_macro` (cases 5, 6) STILL pass — walker is alive through this slice |
| E   | Existing arc 167 tests stay green | tests 1, 2, 3, 4, 7, 8, 9 in `tests/wat_arc167_fn_flat_signature.rs` still pass |
| F   | Lib unit tests in src/runtime.rs preserved | The 793 lib tests still pass; sonnet did NOT touch substrate code |
| G   | Slice 2 + 3 commits all on branch | branch `arc-167-slice-2-fn-sig-consumer` ahead of main with all WIP commits |
| H   | Main untouched | `git log origin/main` unchanged |
| I   | Substrate code untouched | `git diff origin/main..HEAD -- 'src/**/*.rs'` excluding new tests = no substrate-side changes (sonnet operates only on user-code + stdlib + test files) |
| J   | Mechanical translation discipline | spot-check 5 random migrations; each follows the recipe exactly: `((x :T) -> :R)` ↔ `[x <- :T] -> :R` (no creative re-shaping; no semantic changes) |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Sites that don't fit the recipe.** If a wat-source file has a
  fn-sig in an unusual context (string literal in a test that's
  intentionally testing legacy syntax; macro template; etc.),
  report; orchestrator decides whether to migrate or scope out.
- **Comment-only mentions of old shape.** Source comments that
  describe the legacy shape historically (not as live code).
  These should NOT be migrated; report the locations so we know
  they were intentionally skipped.
- **Crate-specific test fixtures with their own quirks.**
  `crates/wat-sqlite`, `crates/wat-telemetry`, etc. have their own
  wat-tests directories. Migrations there are mechanical too,
  but if a crate's test harness has special legacy expectations,
  report.
- **Macro-generated fn-sigs.** If a defmacro template produces a
  legacy-shape fn-sig at expansion time, that's a SUBSTRATE site
  (slice 2/4 territory) — STOP; do not bridge by editing the
  macro template; report and let orchestrator decide whether
  it's slice 4's territory or a substrate gap.
- **Walker false-positive after-the-fact.** If your migration
  triggers a walker firing on a now-correct site (suggesting
  walker has a scoping bug), STOP and report — don't chase it
  with more migrations.
- **`tests/wat_arc166_defn.rs`.** The arc 166 tests use legacy
  fn-sig shape inside their embedded wat strings. Migrating them
  is in scope; verify all 10 of arc 166's tests still pass after
  migration.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed).

Site count by file: ___ (for future-arc prediction calibration).

## What's next (orchestrator-side, post-slice-3)

When slice 3 ships green, orchestrator scores + drafts slice 4.
Slice 4 hard-retires:
- `BareLegacyFnSignature` walker (variant + Display + walker body
  + freeze.rs registration + migration text constants)
- `parse_legacy_fn_signature` + `parse_legacy_fn_signature_for_check`
  (the transitional parser arms)
- `eval_fn` 2-arg legacy arm (if separate from parser arm)
- The dual-arm logic in any places that branched on shape

Slice 4 verifies:
- `cargo test --release --workspace --no-fail-fast`: 0 failed
- `grep -rn 'BareLegacyFnSignature\|parse_legacy_fn_signature' src/`
  returns 0 hits

## SCORE artifact

Sonnet's report writes to chat; orchestrator commits SCORE-SLICE-3.md
to the slice branch after scoring all rows + reviewing the diff.
