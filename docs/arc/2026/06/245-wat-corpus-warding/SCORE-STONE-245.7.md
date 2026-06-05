# SCORE — Stone 245.7: the leak-contained integration runner

Graded against `EXPECTATIONS-STONE-245.7.md`. The run phase was performed BY the
orchestrator (the executor's sandbox denied chmod/exec — an honest stall it
surfaced rather than worked around), so every load-bearing row is first-hand.

## Scorecard

| # | What | Result |
|---|---|---|
| 1 | Leak-safety (THE contract) | ✓ **ZERO surviving test processes** after the full 185-binary run — orchestrator's own before/after snapshot (the "before stray" was the measuring command matching itself; true before = 0) |
| 2 | Completeness | ✓ 185 inventory rows = 185 tier binaries; **0 timeouts** (no gaps, nothing hung). The "4 `[[test]]` entries" in the brief-prep was a miscount of comment mentions — there are exactly 2 (comms, function), both enumerated |
| 3 | Baseline inventory | ✓ `INVENTORY-245.7-baseline.tsv` (snapshotted into this arc dir) + footer |
| 4 | Containment loop = the proven mechanic | ✓ — after an orchestrator FIX (below) |
| 5 | Tier heuristic documented | ✓ script header names it a heuristic; `--all` documented |
| 6 | Blast radius | ✓ `scripts/integration-run.sh` (+ 2 test-file fallout fixes, see below) |
| 7 | Sibling discipline | ✓ green-gate.sh-style header (usage / why-this-exists / heuristic note) |

## Two kills BEFORE the first full run (the weighing earned its keep)

1. **The errexit bug** — the executor's script had `wait "$sid"; code=$?` under
   `set -euo pipefail`; a 5-second probe PROVED it aborts at the FIRST red binary
   (skipping that binary's reap, yielding a 1-line inventory). Fixed to
   `code=0; wait "$sid" || code=$?`, re-proven SURVIVED. Exactly the trap-door the
   Expectations named.
2. **The broken-since-249.5 test-build** — the runner's build-once gate refused to
   start: `cargo build --tests -p wat` had been broken since the 249.5
   encapsulation/ArgSpec changes, invisible because milestones gated on lib +
   targeted probes (the arc-239 lesson recurring: every tests/*.rs is a compile
   unit the lib build never touches). Complete fallout (`--keep-going`): exactly 2
   binaries — `wat_arc170_closure_extraction` (private `.name` → `as_str()`) and
   `probe_arc241_stone1_argspec_canonical` (6× String-assert → `.as_str()`).
   Fixed (`d82ef74b`); test-build CLEAN. **Process recommendation: the milestone
   rhythm should include the test-BUILD** (green-gate check #1), not lib-only.

## THE BASELINE (replaces the stale "~190")

Default tier (185 binaries, 67 leaky-signal excluded):

- **152 binaries pass · 33 fail · 0 timeout** — tests: **1460 pass / 147 fail / 59 ignored**
- Error classes: UnresolvedReference=52 · NoMatchingClause=46 · MalformedForm=17 · TypeMismatch=4 · UnboundSymbol=1
- **Concentration:** 4 binaries hold 91/147 (62%) — `wat_arc148_ord_buildout` (46),
  `wat_arc098_form_matches_{runtime,typecheck}` (15+7), `wat_arc150_variadic_define`
  (14), `wat_core_cond` (9). Long tail: 29 binaries with 1–5 failures each.
- Separate buckets (NOT in this tier): the 67 excluded arc-170 process binaries;
  `crates/wat-holon-lru` (19, struct-rot, named in `94261f45`).
- Newly visible: `wat_arc170_closure_extraction::t12_body_uses_expanded_substrate_primitive_macro`
  (compiled-blind until the fallout fix; plausibly the scoped-param closure-extract
  case 249.5d scoped out).

## Honest deltas

- The executor stalled on sandbox permissions for the run phase and surfaced it
  cleanly (no workaround) — the orchestrator ran verification, which is where the
  weighing lives anyway.
- The "~190" estimate was the right ORDER but the wrong shape: the real triage
  surface is 33 binaries / 147 tests, heavily clustered by arc.

## Disposition

The runner is real, proven leak-safe on the live tier, and the campaign now has a
GROUNDED work-list. Next: the triage — conferre per failing binary (real substrate
gap = fill / stale pre-clojure-ification test = modernize-or-delete), biggest
clusters first (148-ord, 098-form-matches, 150-variadic, core-cond). When the tier
greens: fold the runner into `green-gate.sh` (the #151 endgame) so it can never
rot silently again.
