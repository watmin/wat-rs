# BRIEF 1 — the PILOT: replay grok-rete commits #1–#10 onto main, one at a time

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. **Never use worktrees.** Do not touch
`~/work/holon/` (the frozen root).

## ⛔ TREE STATE

```
branch   replay/grok-rete   <-- YOU ARE HERE (off main a3218644d + tooling + stone 0)
HEAD     (the commit that lands this brief)   floor 5391/5391 · clippy 0 · pushed
source   origin/grok-rete   37528f6e0   FROZEN — read it with `git show`, never check it out
base     de827fb4c          the merge-base: grok-rete commit #1 is the first after it
```

## The goal (the builder's ruling — `SEAM.md` § RULED 2026-09-12)

*"main is upgrading syntax, the others are upgrading subsystems."*

Replay grok-rete's commits **in order, one at a time**, onto this branch. Every rete behaviour grok
built arrives, and every line is in main's syntax.
- **Ownership:** main owns all syntax, plus behaviour outside rete. grok-rete owns rete's behaviour.
  A shared component takes main's version, and grok's change is re-expressed on top of it.
- **Done, per commit:** the commit's own tests exist in main's syntax and pass, at the commit that
  introduces them.

This pilot is commits **#1–#10**. It measures what a step costs, and it builds the step tooling the
remaining 641 (and the next branch) will reuse.

## The ten, measured

| # | commit | code files | the step's kind |
|---|---|---|---|
| 1 | `2186654f7` | `src/rete/kernel/tests.rs` (main changed it) · NEW `tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.{rs,wat}` | .rs conflict + a new `.wat` to convert |
| 2 | `8d73e74b9` | `src/rete/where_tree.rs` (main changed it) | .rs conflict |
| 3 | `084e68192` | `expr_ir.rs` `matcher.rs` `purity.rs` `validate.rs` (main changed all four) | .rs conflicts |
| 4 | `fe301757d` | `src/rete/kernel/arm.rs` (main changed it) | .rs conflict |
| 5 | `051bc9c5b` | `kernel/arm.rs` + 3 files under `kernel/fire/` | .rs |
| 6 | `e0df98193` | `tests/rete/wat_scripts_grid_axes_live.rs` | .rs |
| 7 | `15dcca1df` | NEW `tests/rete/probe_arc278_stratified_query_replay.{rs,wat}` | a new `.wat` to convert |
| 8 | `2615e94a5` | docs only | cherry-pick |
| 9 | `afb58d422` | 12 files, including `tests/rete/probe_arc278_7exists_native_differential.wat` (grok-only change) and **`wat/cache.wat` (main changed it too)** | .rs + a `.wat` 3-way |
| 10 | `26a0d937a` | `src/rust_deps/cache.rs` + **`wat/cache.wat`** | a `.wat` 3-way |

## Build the step tooling FIRST — tracked, repeatable, in `scripts/replay/`

1. **`scripts/replay/chain-order.sh <base> <main>`** DERIVES the chain from git: the codemods added
   under `wat-scripts/fixes/` between `<base>` and `<main>`, in landing order. It then applies
   `scripts/replay/chain-order.overrides` (a tracked file of moves, each line carrying its reason).
   - The one override today is **PROVISIONAL**: `bare-variant-to-qualified` runs BEFORE
     `positional-ctor-to-map`. `49f03f179` rewrote positional-ctor to match qualified names only.
     Confirmed: a bare `(:wat::core::Some 1)` stays positional under landing order.
   - Its line says PROVISIONAL, pending the orchestrator's chain-composition check (running now,
     on copies).
   - Check: the derived list, before the override, equals `bootstrap/landing-order.txt`'s 27 lines.
2. **`scripts/replay/convert.sh <rev> <path>`** writes to stdout `<path>` at `<rev>`, converted
   through that chain by the tree's own `target/release/wat` (built at HEAD).
   - Each codemod runs only if `<path>` is inside its `;; SCOPE:`.
   - It works in a temp dir, never on the tree.
   - A nonzero rc from any codemod is fatal, and prints the whole log.
   - Deterministic: two runs, byte-identical. Prove it on one file.

## One step

For grok-rete commit **C**, where #N is its index after `de827fb4c`:

1. **Docs-only:** `git cherry-pick -x C`.
2. **Otherwise:** `git cherry-pick -x --no-commit C`, then per file:
   - **`.wat` new in C:** the result is `convert.sh C <path>`.
   - **`.wat` modified in C:**
     `git merge-file <working copy> <(convert.sh C^ <path>) <(convert.sh C <path>)`. The working
     copy is main's version, already in main's syntax. The converted before/after carry grok's change
     into main's syntax. A conflict resolves by the ownership rule.
   - **`.wat` deleted in C:** `git rm`.
   - **`.rs`:** resolve the conflicts by the ownership rule. Where main moved a home or changed a
     signature, main's version stands and grok's change is re-expressed on it. The first merge's
     per-hunk record is a crib, not an authority. It exists only on `merge/grok-rete`, so read it
     with `git show merge/grok-rete:docs/arc/2026/06/294-holon-returns-to-vsa/the-grok-rete-merge/SCORE-merge-grok-rete.md`
     (never check that branch out).
   - **wat embedded in `.rs` strings:** bring it to main's syntax by hand. The codemods do not reach
     it. Log each edit.
3. **Gate the step:**
   - `cargo build --release` passes;
   - every `.wat` the step produced passes `./target/release/wat --check`, or is a deliberate `.bad`;
   - **the tests C adds or changes** are run by name (`cargo nextest run --release -E 'test(<name>)'`)
     and pass.
4. **Commit:** `REPLAY(grok-rete #N): <C's subject>`, whose body carries C's hash, every conflict,
   and how each was resolved. `-x` adds the trailer.
5. **Log it** in `PILOT-LOG.md` (this directory): N, C, the wall time, the files, the conflicts and
   their resolutions, the hand edits, the tests run and their result, the tricks learned.

**At #5 and #10:** `scripts/floor.sh` (read the Summary) and `cargo clippy --release --all-targets`.
The orchestrator's composition check runs concurrently at nice 19, on copies only. If a floor goes
red, report it verbatim and **do not re-run**. The orchestrator checks whether contention was in
play.

## ⛔ STOP triggers — each is a REJECTION. Commit nothing further; report the verbatim evidence.

- **STOP-1:** C's own tests fail after the step, and the cause is not syntax. Rete behaviour did not
  survive. This is the thing the whole replay exists to prevent.
- **STOP-2:** a converted `.wat` fails `--check` or its load gate because of the CHAIN (a codemod
  missed or broke something). Do not hand-fix the `.wat` (R21). Name the codemod and paste the
  whole output.
- **STOP-3:** an `.rs` conflict needs a THIRD behaviour, on neither side.
- **STOP-4:** a checkpoint floor is red for a reason inside the replayed commits' files.
- **STOP-5:** `convert.sh` is not deterministic, or `chain-order.sh`'s derived list disagrees with
  the 27.

## Blast radius

The replayed commits' own files, `scripts/replay/`, and this directory's `PILOT-LOG.md` and SCORE.
`wat-scripts/fixes/` stays untouched except where a replayed commit itself changes a codemod (none of
#1–#10 does).

## Tier

Commit each replayed step on green. **Do not push. Do not touch main.** Yield after #10, or at the
first STOP, with `SCORE-1-pilot.md` and `PILOT-LOG.md`.
