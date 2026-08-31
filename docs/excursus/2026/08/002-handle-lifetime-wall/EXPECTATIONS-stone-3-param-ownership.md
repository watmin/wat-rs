# EXPECTATIONS — excursus 002 stone 3

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | road 3 is rejected | `--check probes/red-param-tail-escape.wat` | REJECTED, naming `:red::drive-param` and the service |
| 2 | upward is UNTOUCHED | same run | `:red::conn` NOT named. If it is, the widening leaked across directions and every `conn` helper in the corpus dies |
| 3 | the binding form compiles | same run | `:red::held-param` NOT named — the drive is in a binding, so the frame outlives it |
| 4 | stone 1 still holds | `cargo nextest run --release -E 'test(probe_ex002_creation_escape)'` | 1 passed |
| 5 | stone 2 still holds | `cargo nextest run --release -E 'test(probe_ex002_tail_escape)'` | 1 passed |
| 6 | census, RUN | build, `--check` every corpus `.wat` | rejects ONLY the new red probe + already-runed instruments. **Any live-code hit is a STOP, not a rune** |
| 7 | the bisect still RUNS | `./target/release/wat wat-scripts/scratch-pad/probe-self-sched-bisect.wat` | prints its table, `C-param-tail` row included |
| 8 | no runtime change | `git diff --stat src/runtime.rs` | empty |
| 9 | no rune on the criterion | `grep -n '^\s*;; rune:' probes/red-param-tail-escape.wat` | none. **Match the FORM, not the token** — prose about runes is not a rune (I got this wrong twice grading stone 2) |
| 10 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5136+ run, 0 failed, FLOOR=0 |
| 11 | the error teaches | read the rejection | names function, service, the param that owns the handle, and the tail-call span |
| 12 | the four-cell table holds | the three probes together | upward-param legal; downward-create, downward-param, upward-create all rejected |

**Runtime prediction:** 30–60 minutes. The rule is one predicate widened; most of the cost is the
census and the three-shape probe.

## Trap doors, named in advance

- **Widening both directions.** The single most likely failure, and it kills `conn` corpus-wide.
  Row 2 is the only thing standing there.
- **Firing on nothing.** Rows 2–5, 7, 8, 10 all pass for a wall that never fires. Only rows 1 and 6
  catch it. A green floor with no rejection is a FAILURE.
- **Runing a live-code hit** instead of reporting it. Row 6 is a STOP, not a cleanup task: a common
  false positive means the trade is wrong and the stone should not ship.
- **Reaching for `src/runtime.rs`.** TCO is not the defect and is not being fixed.
