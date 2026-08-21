# EXPECTATIONS — STONE 255.1c-io-fs

Written BEFORE the strike. The rider does not see this file.

| # | what | expected |
|---|---|---|
| 1 | ★ the family is fully carved | `grep -c '":wat::io::[^"]*" *=>' src/runtime.rs` → **0** |
| 2 | the foreign family survived | `":wat::stdlib::sources"` → 1 |
| 3 | six rows | 6 |
| 4 | registry 120 → 126 | 126 |
| 5 | checker untouched | empty diff |
| 6 | tests untouched by the rider | empty diff |
| 7 | builds | clean |
| 8 | five registry gates | 5/5 |
| 9 | ★ the gate can FAIL | perturb `TempFile/path`'s `@ret` String→i64 → RED; revert |
| 10 | probe population still 29 | `29 · 28 · 1 · 0` |
| 11 | the rete completeness gate | PASS — population grows again, +6 |
| 12 | floor | 4818/4818, 0 FAIL |
| 13 | clippy | 0 |
| 14 | rustfmt parity | `io/fs.rs` clean |
| 15 | goldens | 25209 → measured |

★ Row 11 is new and it is the point. The gate I repaired at `85174fc3f` now takes the UNION of
`runtime.rs` arms and registered intrinsics. This strike moves six more verbs across that boundary —
if the union is right the population is unchanged in total and the gate stays green. **A red here
means my fix was wrong**, not that the rider erred.

## The goldens

Five fixtures pin `:line 25209 :col 17`. The cut at 6453 is above them.
Predicted: 6 arms × 3 lines = 18, plus the comment block rewrite → **25209 − ~18±4**. Measure from the
hunks, never `--numstat`.

## Independent prediction

**Runtime: 12–20 min.** Six rows, uniform arg order, two prior templates on disk. Strike 2 did
thirteen rows in 10 min.

## Trap doors

- **`:wat::stdlib::sources` sits directly below the six.** A greedy range deletion takes it. Row 2.
- **Both `*/new` verbs are nullary.** Zero `@arg`. The gate's `i < params.len()` guard is 0 there, so
  an invented `@arg` is silently skipped, not caught. Count by reading `grep -n '@arg'` output.
- **`TempFile/path` and `TempDir/path` both return `String`** while their `*/new` siblings return the
  handle type. Adjacent rows, different returns — the same shape that made `writeln` the perturbation
  target last strike.
- **Category over-fitting.** Six rows under a file called `fs.rs` invite six `:Io`s. The `*/path`
  verbs look like `:Projection`; `read-file`/`list-dir` open nothing the caller keeps. If the column
  comes back uniform, re-read the bodies myself.

## What would make me reject

`src/check.rs` diffs · `stdlib::sources` moved · a `@Category` whose comment quotes no `src/io.rs`
line · six identical categories · row 9 failing to go red · row 11 red.
