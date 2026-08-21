# EXPECTATIONS — STONE 255.1c-io-reader

Written BEFORE the strike. The rider does not see this file.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the ten arms are gone from `runtime.rs` | `grep -c '":wat::io::IOReader/[^"]*" *=>' src/runtime.rs` | `0` |
| 2 | the thirteen writer arms are untouched | `grep -c '":wat::io::IOWriter/[^"]*" *=>' src/runtime.rs` | `13` |
| 3 | ten rows registered | `grep -cE '^\s*#\[wat_intrinsic\(' src/intrinsic/io/reader.rs` | `10` |
| 4 | registry total moved 97 → 107 | `grep -rhcE '^\s*#\[wat_intrinsic\(' src/intrinsic/ \| paste -sd+ \| bc` | `107` |
| 5 | the checker was NOT edited | `git diff --stat src/check.rs` | empty |
| 6 | tests were NOT edited | `git diff --stat tests/` | empty |
| 7 | `stdlib::sources` survives the cut | `grep -c '":wat::stdlib::sources"' src/runtime.rs` | `1` |
| 8 | it compiles | `cargo build --release` (ORCHESTRATOR) | clean |
| 9 | the doc/scheme gate is green **and non-vacuous** | `nextest -E 'test(doc_arg_ret_types_match_checker_scheme)'` | pass, after row 10 |
| 10 | ⚠ **row 9 can pass by SKIPPING.** Perturb one `@ret` and confirm it goes RED | edit one `@ret`, re-run row 9, revert | RED, then green |
| 11 | every io verb still checks | `PROBE-255.1c-io-every-verb-is-scheme-enforced.sh` | 28 / 1 / **0** — unchanged |
| 12 | the floor | `scripts/floor.sh` (ORCHESTRATOR) | 4818/4818, 0 FAIL |
| 13 | clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
| 14 | the goldens | see below | 5 fixtures bumped by the measured delta |

★ **Row 10 is the load-bearing row, not row 9.** `doc_arg_ret_types_match_checker_scheme` opens
`None => continue` and its arg loop is guarded by `i < scheme.params.len()`. A green run therefore
proves nothing until a deliberate perturbation is shown to turn it red — a gate that cannot fail
verifies nothing (R59 `NISI FRANGAS, NIHIL PROBAS`), and this campaign has already shipped one
"14/14 PASS" that exercised four rows.

## The goldens — the standing orchestrator step

Five fixtures pin `src/runtime.rs :line 25277 :col 17`. The cut is at 6448, entirely above them.

1. `git diff src/runtime.rs` — read the hunks, do **not** read `--numstat`.
2. Confirm **every** hunk precedes 25277. (A prior stone's net was −64 while only −12 sat above the
   pinned site; the net is not the delta.)
3. New line = 25277 − (lines removed above it) + (lines added above it). Prediction: the rider
   deletes 30 arm-lines and rewrites a 4-line comment, so **−30 ± 2** → `25247`, ±2.
4. `:col` stays `17`. Only `:line` moves. Then the floor.

## Independent prediction

**Runtime: 25–40 min.** Ten rows, each needing a body-read in `src/io.rs` and a scheme transcription
from `check.rs` — slower per row than `kernel/stdio.rs`'s six (which shared one delegate shape), but
the rooms are mapped to the line and the template is on disk.

## Trap doors, named before the strike

- **The delegate arg-order split.** Four verbs pass `(args, list_span, env, sym)`, six pass
  `(args, env, sym, list_span)`. Both compile as far as the rider can see without building, and the
  rider is not building. **The orchestrator's build is the only thing that catches a swap** —
  and `env`/`sym` are different types from `list_span`, so it will. Check this in the diff by eye too.
- **`read-frame`'s `@ret`.** Its scheme's `ret` is a `ReadFrameOutcome`, not the `opt_string_ty()` a
  reader might assume from `read-line` two entries above it. Highest-probability single failure.
- **Category over-fitting.** The likeliest wrong answer is labelling all ten `:Io` because the family
  is called io. `from-bytes` / `from-string` take a value and hand back a value with no syscall; the
  design predicts they are NOT `:Io`. If the rider returns ten identical categories, that is the
  signal to re-read the bodies myself, not to accept it.
- **A silent scheme skip.** If the rider omits an `@arg` past `scheme.params.len()`, row 9 stays
  green and the doc is incomplete. Row 10 does not catch this either. Count `@arg` lines per row
  against the scheme's `params.len()` by hand.

## What would make me reject this strike

Any of: `src/check.rs` shows a diff; a `@Category` whose `//` comment quotes no `src/io.rs` line;
ten identical categories; or row 10 failing to go red under perturbation.
