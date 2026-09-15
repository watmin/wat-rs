# BRIEF 5 — replay batch 2: grok-rete #61 → #125

> Built on batch 1 (closed and pushed), 2a4c (the stdlib door replaces a divergent macro; STOP-9),
> 4b (the per-step census; STOP-8) and stone 3 (the nested-program gate; STOP-10). Released after
> stone 3 closed.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees.** Do not touch `~/work/holon/` or `main`.

## The work

Replay grok-rete commits **#61 through #125** (`#61 = 635895348`), one at a time, in order, by
`BRIEF-1-pilot-first-ten-commits.md` § "One step" — including its rows added by 4b and 2a4c: the fast
lint subset on every step with a `.rs` change, the whole-tree `census.sh` + `--diff` whenever the
binary or any `.wat` changed (STOP-8), and `convert.sh` refusing a `wat/` file (STOP-9).

**Stone 3's gate rides with the census, not the lint subset.** A child program's verdict changes
exactly when the census's can — the binary or a `.wat` changed — and the census's `--check` never
starts a child. So on every census step also run
`cargo nextest run --release -E 'test(nested_program_literals_start_on_the_child_path)'` (29 s
isolated), and drop it from the lint subset:
`-E 'binary(lint) - test(every_wat_scripts_file_loads_on_the_current_runtime) - test(nested_program_literals_start_on_the_child_path)'`.
A red is **STOP-10**: a nested child that started before this step no longer starts.

The census for this range (`bootstrap/era/replay-plan/commits.tsv`): 30 docs-only, 13 code touching only
grok-rete files, 22 touching a file main also changed; 233 files, 98 of them `.rs`. **None touches
`wat/`** (so STOP-9 cannot fire), `wat-scripts/fixes/`, a file main deleted, or positional-ctor's
skip-listed paths (the first such is #379). The census baseline for #61 is `.census/latest` at the
tip you start from.

## Checkpoints

After **#93** and after **#125**: `scripts/floor.sh` (read the Summary line) and
`cargo clippy --release --all-targets -- -D warnings`. A red floor is reported verbatim and never
re-run.

## The log

`REPLAY-LOG.md`, one row per step as in batch 1, plus: the census diff's verdict for every qualifying
step, and the lint subset's for every `.rs` step.

## ⛔ STOP triggers — each is a REJECTION. Commit nothing further; report the verbatim evidence.

- **STOP-1 … STOP-5:** as `BRIEF-1-pilot-first-ten-commits.md` § "STOP triggers".
- **STOP-6:** a step touches `wat-scripts/fixes/`, a file main deleted, or a `wat/` file — the census
  says none in this range does.
- **STOP-7:** the LATENT rule has no registered verb of the same meaning (`BRIEF-4-ADDENDUM-latent-defects.md`).
- **STOP-8:** the census diff names a file going rc 0 → non-zero that the step did not produce.
- **STOP-9:** `convert.sh` reports `UNREGISTERABLE` for a `wat/` path.
- **STOP-10:** stone 3's gate is red at a census step (paste its whole block, do not re-run).

## Tier

Commit each replayed step on green: `REPLAY(grok-rete #N): <C's subject>`. **Do not push.** Yield after
#125, or at the first STOP, with `SCORE-5-replay-batch-2.md` and `REPLAY-LOG.md`.

## Expectations

`EXPECTATIONS-5-replay-batch-2.md` (written before the strike).
