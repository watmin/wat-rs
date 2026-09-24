# BRIEF — STONE D: shard the wat-scripts load gate

Read `DESIGN-stone-D-shard-the-wat-scripts-gate.md` first.

## The work

Split `every_wat_scripts_file_loads_on_the_current_runtime` into `N_SHARDS` stride shards exactly
as `every_wat_bad_fixture_actually_fails.rs` does, keeping the original name as every shard's
PREFIX, and resize the three nextest overrides that currently give the single test 1500s/3000s.

## Rooms

1. **`tests/lint/wat_scripts_fixes_load.rs`** (71 lines, the whole gate) — keep `collect_wat`, the
   `FsLoader`, the per-file verdict and the failure text byte-for-byte in meaning.
2. **`tests/lint/every_wat_bad_fixture_actually_fails.rs`** — the precedent. `N_SHARDS` (`:94`),
   `check_shard` (`:296`) with its `skip(shard).step_by(N_SHARDS)` and its per-shard non-empty
   assert (`:311-317`), and the generated shard tests (`:393-408`). **Copy the shape.**
3. **`.config/nextest.toml`** — the wat-scripts override appears **THREE times**
   (`[profile.default]`, `[profile.ci]`, `[profile.slow]`) at 1500s/3000s + `priority = 100`.
   ⛔ Overrides do NOT inherit — all three must move together. Its long comment block records the
   gate's history; append the split to it rather than deleting the history.
4. **`tests/lint/every_walking_gate_declares_non_vacuity.rs`** — this gate JUDGES yours (the file
   contains `read_dir`). Keep a `NON-VACUITY` marker with an assert within 12 lines below it.

## Sketch

```rust
const N_SHARDS: usize = /* derived — see the DESIGN's invariant */;
fn check_shard(shard: usize) -> Vec<String> { /* walk, floor-assert, stride, non-empty assert, verdicts */ }
// one generated #[test] per shard: every_wat_scripts_file_loads_on_the_current_runtime_shard_NN
```

## Blast radius

`tests/lint/wat_scripts_fixes_load.rs` · `.config/nextest.toml`. Nothing else.

## STOP triggers

1. **A shard's measured cost violates the invariant** at the N you chose — re-derive N, do not
   raise a budget to cover it.
2. **The corpus-floor or a shard non-empty assert cannot be written without going vacuous.**
3. **Any gate reddens that you did not add.** Capture whole, name the arm, report. ⛔ Do not re-run.

## Mechanics — ⛔ read this, the last three executors each lost time here

- **The Bash tool caps at 600s and backgrounds anything longer**, whatever an instruction says. A
  floor cannot be foregrounded. What works: `nohup scripts/floor.sh > <file> 2>&1 &`, then
  repeated FOREGROUND blocks of `until grep -qE '^ *Summary' .floor/latest/clean.log; do sleep 30;
  done` (each under 600s) until it lands. The result stays owned by you. **Do not end your turn
  while it runs.**
- Never poll with `pgrep -f 'cargo …'` — it matches its own command line.
- Read the `Summary` line, never a piped exit code.
- `cargo fmt` reformats the WHOLE workspace. Use `rustfmt <file>` or neither.
- `git add` BEFORE running `git ls-files`-based gates.

## Prove it

- **Mutation 1:** break one file under `wat-scripts/` (a stray unparseable form in a scratch copy
  you then delete) — exactly ONE shard reds, naming that file. Restore.
- **Mutation 2:** set `N_SHARDS` so one shard is empty (or blind the walk root) — the non-vacuity
  assert reds. Restore.
- Report: N, the per-shard isolated time you measured, the arithmetic against the invariant, and
  the floor's wall-clock before (~1026 s) and after.
