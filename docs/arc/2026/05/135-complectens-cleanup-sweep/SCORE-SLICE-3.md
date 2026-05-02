# Arc 135 Slice 3 — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `afcdd55e55ebb7dec`
**Runtime:** ~24 min (1435s).

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Two-file diff | **PASS** | Only `WorkUnit.wat` + `WorkUnitLog.wat` modified. |
| 2 | Helpers added | **PASS** | 6 new helpers (3 wu-spawn-stub-scope-* + 3 wul-* helpers). |
| 3 | Each existing deftest body shrinks | **PASS** | All 7 deftests at 4-7 outer logical bindings (target band). The largest before-after delta: WorkUnit's deftests (62/57/56/55/47 visual lines) all hit 4-7 outer logical. WorkUnitLog's deftests already had 4 outer bindings; the violation was the inner let* — collapsed into the body lambda fixture per SKILL edge case 7 (embedded literals). |
| 4 | Per-helper deftests added | **PASS** | 4 new per-helper deftests. Several helpers (`wu-recv-metric-uuid-ok`, `wul-extract-level`, `wul-recv-level`) exempted as Level 3 taste with documented reasons (cannot construct synthetic Event::Log fixtures without substrate-internal fields). Same pattern as slice 1 + 2's RunResult exemption. |
| 5 | No forward references | **PASS** | Top-down: prelude → per-layer deftests → scenarios. |
| 6 | **Outcomes preserved** | **PASS** | `cargo test --release --workspace` exit=0; wat-telemetry 38→39 (one new clean pass for `test-wul-spawn-stub-and-emit-drain`); zero regressions. |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications. |
| 8 | Honest report | **PASS+** | Substantive Delta 1 surfaced — substrate limitation on generic-T 3-tuple returns. Workaround documented (three concrete non-generic helpers with nested 2-tuple returns). Same pattern surfaced in slice 2's Delta 3 (cadence type unification cost) — this is a recurring substrate observation, NOT a SKILL gap. |

**HARD VERDICT: 8 OF 8 PASS.**

## Soft scorecard (4 rows)

| # | Criterion | Result |
|---|---|---|
| 9 | Helper count | PASS — 6 helpers across the largest slice. In band. |
| 10 | Average body shrink | PASS — outer logical 4-7 across all 7 deftests. |
| 11 | Workspace runtime | PASS. |
| 12 | Edge-case usage | PASS — sonnet correctly applied SKILL edge case 7 (embedded literals — body lambda is fixture, not scaffolding). |

**SOFT VERDICT: 4 OF 4 PASS. Clean ship.**

## Substrate observation — generic-T 3-tuple return doesn't propagate

Sonnet's Delta 1: an attempted helper signature `(define :helper<T> body -> :(Thread, T, Receiver))` returned T at runtime instead of the 3-tuple. Workaround: three concrete non-generic helpers with nested 2-tuple returns `(Thread, (T-concrete, Receiver))`.

This is a **substrate bug or limitation**, NOT a discipline issue. Same observation surfaced in slice 2's Delta 3 (cadence-type unification cost). Both point at type-inference quirks for generic Ts in multi-element tuples.

**Action:** file as a separate substrate concern. NOT a SKILL refinement. Likely lives in arc 109 J-PIPELINE follow-ups OR opens a new substrate arc when prioritized. For now, the workaround (concrete helpers + nested 2-tuples) is the documented pattern.

## Independent prediction calibration

This was the largest slice in the queue. The teaching cascade compounded: slice 1 surfaced 3 deltas → SKILL refinement → slice 2 surfaced 1 delta → SKILL refinement → slice 3 surfaced 1 delta (substrate-level, not SKILL-level).

Sweep timing: 24 min. Larger than slice 1 (16 min) and slice 2 (8 min). Bigger surface (5 + 2 = 7 deftests; 6 new helpers) explains the additional time.

The new pattern: each slice's deltas may now point at SUBSTRATE quirks rather than SKILL gaps. The discipline is settled; remaining gaps are in the substrate.

## Ship decision

**SHIP.** 8/8 hard + 4/4 soft. Substrate-level deltas filed for separate disposition.

## Next steps

1. Update `arc-130/FOLLOWUPS.md` — mark WorkUnit.wat + WorkUnitLog.wat ✓.
2. Commit slice 3 + this SCORE.
3. Spawn slice 4 (suspect-tier — wat-tests/test.wat + stream.wat + HologramCache.wat + step-B-single-put.wat). Phase-2 judgment for each — REFACTOR or EXEMPT.
