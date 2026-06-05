# EXPECTATIONS — Stone 249.3a — eval-time quasiquote: purity fence + `~@`-splice + `List?` predicate

**Brief:** `BRIEF-STONE-249.3a.md`. **Design:** `DESIGN-STONE-249.3.md` §2.1 + §2.2 + §2.3.

## Independent prediction (orchestrator, pre-spawn)

- **Runtime band:** 15–22 min (Mode A). Three located changes in well-precedented code: the fence extends one validator function (`validate_pure_total`) with depth-tracked unquote descent; the splice ports the existing expand-time `splice_argument` semantics into `walk_quasiquote`; the `List?` predicate is a small impl + dispatch arm + allow-list entry (mirrors `record?`). No new files, no new error variants.
- **Time-box:** 2× upper = **45 min** → `ScheduleWakeup` at +45 min; if still running at wake, `TaskStop` + score Mode B-time-violation.
- **Risk surface:** (1) depth tracking in the fence's quasiquote descent (mirror `walk_quasiquote`'s existing depth logic exactly); (2) the outer-list flatten for splice (1-to-N at the parent list level, not a nested node); (3) the `wat__WatAST(List)` splice semantic (splice the inner list's *children*); (4) the `List?` source-divergence comment (don't omit — intueri Level-1).

## Scorecard methodology (how I score on return)

Verify each row by re-running locally — do NOT accept self-report (FM-9):

| Row | Command | Pass = |
|---|---|---|
| Purity fence | `probe_arc249_threading_in_wat` row E (un-ignored) | startup REFUSED (impure unquote rejected) |
| Splice thread-last | `probe_arc249_threading_in_wat` rows A, B (un-ignored) | green (`~@step` splices) |
| `List?` predicate | `probe_arc249_threading_in_wat` row C (un-ignored) | green (true for list form, false for int) |
| Engine contract | `probe_arc249_macro_engine` gates A–E | all green (no regression) |
| Library | `cargo test --release --lib -p wat` | pass count holds (≥ 898/0/1) |
| Clippy | `cargo clippy --release -p wat` | zero new warnings on touched lines |

Row D is a diagnostic (leave as-is).

## Path-honesty check (FM-9 corollary)

Row E must measure PURITY (a type-compatible impure unquote refused), not a type coincidence — the probe already encodes the type-compatible form (`(not ~(stopped?))`). Confirm the refusal is `RefusedInMacro` (the fence fired), not a downstream `TypeMismatch`/other. If the fence "passes" via the wrong mechanism, it is not closed.

## On completion

- Commit 249.3a-i atomically (BRIEF + EXPECTATIONS + the two src edits + the un-ignored probe rows) once the scorecard is green — orchestrator commits, not sonnet.
- Then: spawn 249.3a-ii (the intueri-named form-shape predicate → row C green) and circumspicere the closed engine (it found F5; survey the fence + splice).
