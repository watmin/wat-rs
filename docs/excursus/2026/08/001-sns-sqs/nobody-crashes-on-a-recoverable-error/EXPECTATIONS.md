# EXPECTATIONS — nobody crashes on a recoverable error (the census)

Written **before** the strike. Graded on the orchestrator's **own** re-runs.

Baseline (`b70c4f8ad`): floor **5251**/5251, 0 FAIL, **538.417 s** (orchestrator's quiet box), clippy 0.

⚠ **The executor's floor wall clock is not comparable** — six stones running: executor
+29.2/+29.6/+18.0/+23.2/+27.2/+22.0 s where the orchestrator measured +7.5/+0.96/+0.24/+0.025/+1.6/+1.0 s
on the same trees. Report it; the orchestrator's run grades row 7.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⭑⭑ **every new family has BOTH controls** (STOP-1) | the SCORE | one PRESENT and one ABSENT control **named per family**. Missing either ⇒ the census is a number, not a finding |
| 2 | ⛔ **the D5 instrument still passes its own controls** | run it unchanged | CONTROL A ABSENT, CONTROL B PRESENT, numbers as cited in D5's SCORE |
| 3 | ⭑ **exemplars, not totals** | the SCORE | ≥1 real `file:line` row per (direction × family) bucket |
| 4 | ⭑ **the head/home split is reported** | the SCORE | live-service arms separable from probes/tests/scratch. D5 was 544 total vs ~59 in scope |
| 5 | ⛔ **`Stopped` is its own bucket** | the report | reported separately, never folded into the recoverable set |
| 6 | ⛔ **`service.wat:2549` is FLAGGED, not ranked** | the SCORE | appears in the census AND is marked unreachable-by-unknown-kind, per the FINDING |
| 7 | floor | `./scripts/floor.sh` → **Summary** | `5251 passed` or higher, 0 FAIL. A SHRINK is a finding |
| 8 | clippy | `cargo clippy --all-targets -D warnings` | 0 |
| 9 | tests compile | `cargo nextest run --release --no-run` | clean |
| 10 | ⛔ **nothing was migrated** | `git status --porcelain` | one census file under `wat-scripts/`. 0 `src/`, 0 `wat/`, 0 service scripts |
| 11 | direction is DERIVED | the SCORE | from the variant family, not guessed per-site |
| 12 | scope stated | the SCORE's headline | a census was taken. Nothing was made tolerant |

## Runtime prediction

**60–90 minutes.** The filter is one predicate; the cost is the controls (one pair per family) and
writing the report so it is rulable rather than merely large.

## Trap-door risks, ranked

1. ⭑⭑ **Row 1 skipped.** A widened census with D5's old controls proves only that the *old* variant
   still works. Every new family needs its own, or the new numbers are unvalidated — which is the exact
   failure mode behind five wrong claims this session.
2. **Row 10 non-zero.** Migrating "just one obvious arm" makes the census un-rulable and pre-empts the
   builder's sequence.
3. **A total with no head/home split.** 544 vs ~59 is the difference between an alarming number and an
   actionable one.
4. **`:2549` ranked as work.** It is live code with a fatal body that nothing known reaches, and which
   kind of unreachable is unanswered. Ranking it repeats how arc 259's ruling once sat above real work.
