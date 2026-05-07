# Arc 145 — Typed Let Consumer Migration EXPECTATIONS (sweep 1b)

**Drafted 2026-05-06.** Pre-handoff scorecard for sweep 1b of
arc 145's typed-let work.

**Brief:** `BRIEF-CONSUMERS.md`
**Output:** EDITS to all wat call sites of `let` / `let*` to
add `-> :T` at HEAD; iteration via cargo test until 0 failed.
NO substrate-side edits. NO commits.

## Setup — workspace state pre-spawn

- HEAD: `e173bd5` (BRIEF + EXPECTATIONS for sweep 1a)
- Working tree dirty with sweep 1a substrate edits (4 files)
- Pre-baseline: 652 passed / 129 failed / 0.34s
- Failure shape: uniform MalformedForm migration-hint on let/let*
- New arc145 tests: 5/10 pass (5 fail under sweep-1a isolation)

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Workspace 0 failed | `cargo test --release --workspace` returns 0 failed across all crates |
| 2 | Arc145 tests 10/10 | `cargo test --release --test wat_arc145_typed_let` shows 10 passed / 0 failed |
| 3 | No substrate-side edits | No changes to `src/*.rs` (sweep 1a's territory) |
| 4 | No `:Any` injections | grep for `:Any` returns no new instances |
| 5 | No commits | `git log --oneline` shows the same commit as pre-spawn (e173bd5) |
| 6 | Sweep order respected | stdlib (`wat/*.wat`) migrated before tests |
| 7 | Migration-hint resolution | every original MalformedForm migration-hint resolved (no remaining "now requires `-> :T`" in cargo output) |
| 8 | No unexpected substrate red | only Mode-A diagnostic kinds (MalformedForm migration-hint, body/recipient TypeMismatch) surfaced; no panics, parse errors inside substrate, or unrelated runtime crashes |
| 9 | Convergent `:T` honest | Distribution of declared `:T` makes sense — many `:unit` for binding chains; concrete types where bodies produce them; no `:Any` cheats |
| 10 | Honest report | Per BRIEF reporting requirements (crawl, sweep summary, T distribution, iterations, verification, path, deltas) |

**Hard verdict:** all 10 must hold. Rows 1, 2, 7, 8 are the
load-bearing rows (workspace converges via substrate-as-teacher
without escape hatches).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | Iteration count | 3-8 cargo test runs to converge (substrate-as-teacher pattern; first run resolves bulk via `:unit`; later runs narrow). >12 = grinding signal. |
| 12 | Per-site grinding | No single site required >3 iterations to converge (per BRIEF "no grinding" constraint). Surface as Mode-D if hit. |
| 13 | Files-touched count | 100-200 .wat files + 30-65 .rs files (embedded wat strings). Outside this band = scope error. |
| 14 | Wall-clock | 60-120 min within time-box; >120 = Mode B-time-violation. |

## Independent prediction

- **Most likely (~60%) — Mode A clean.** Substrate-as-teacher
  has shipped successfully across arcs 109/111/112/113/114; the
  diagnostic stream teaches cleanly per site. ~80-100 min wall-
  clock; ~5 cargo iterations.
- **Mode B-substrate-internal-bug (~10%):** an edge case in
  sweep 1a's `bindings_idx` defensive fallback (deadlock walkers)
  or step_let_star surfaces a panic on some specific consumer
  shape. Honest STOP + report.
- **Mode C-unclear-diagnostic (~15%):** a class of sites where
  the substrate's diagnostic doesn't make the migration obvious
  (e.g., a parametric let whose `:T` should be a free type
  variable but the diagnostic suggests a concrete type). Surface
  the gap; orchestrator may need to amend the substrate or
  manually flag those sites for human review.
- **Mode D-grinding-cap (~10%):** sweep converges but a few
  sites required >3 iterations. Surface; possibly a signal that
  the substrate's diagnostic for those sites needs more
  teaching.
- **Mode B-time-violation (~5%):** sweep doesn't complete in 120
  min. Surface progress; orchestrator decides whether to extend,
  split, or re-brief.

## Time-box

120 minutes wall-clock. ScheduleWakeup at T+120 min (= 7200s).
On wake-up: if sonnet still running, TaskStop + Mode B-time-
violation in SCORE. Else: read sonnet's report and score.

## What sonnet's success unlocks

**Mode A clean:** workspace returns to 0-failed; orchestrator
commits sweeps 1a + 1b + SCORE docs atomically; arc 145 slice 2
(closure paperwork — INSCRIPTION + 058 row + USER-GUIDE) ships
next; arc 145 closes; arc 136 (typed do, Option B per 2026-05-
06) spawns.

**Mode B/C/D:** surface gap; orchestrator adjusts brief or
substrate; reland.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Sample-verify by re-running the canonical commands locally:
  - `cargo test --release --test wat_arc145_typed_let` (must show 10/10)
  - `cargo test --release --workspace 2>&1 | grep "test result:"` (must show 0 failed)
- Sample 2-3 migrated call sites to verify the convergent `:T`
  matches what BOTH body and recipient would expect (the four-
  questions Honest check).
- **THEN COMMIT** atomically with sweeps 1a + 1b + the two SCORE
  docs (or score + commit + then write SCORE-SLICE-2.md as part
  of closure paperwork).

## Why this matters

User direction 2026-05-06: "typed let then do" + "do is value
bearing, so it should be typed" + ":unit is fine with me - it'll
probably be correct in a few places". This is sweep 1b — the
consumer migration that ships the REQUIRED `-> :T` discipline
across every existing call site. After arc 145 closes, arc 136
(typed do) ships per the same pattern.

The mutual-agreement chain:
- User → Orchestrator: "typed let then do" + ":unit is fine"
- Orchestrator → Sonnet (this brief): substrate-as-teacher loop;
  iterate cargo test until 0-failed; convergent `:T` honest
- Sonnet → Reality: workspace returns to 0-failed; sweep 1b
  ships; atomic commit with sweep 1a; arc 145 closes

Mode A clean = the typed-let discipline holds across the entire
substrate; the foundation gains a cleanly-typed control flow
form; arc 109's "no bridges" doctrine extends.
