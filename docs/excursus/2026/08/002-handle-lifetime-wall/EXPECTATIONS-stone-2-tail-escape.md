# EXPECTATIONS — excursus 002 stone 2

Written BEFORE the strike. Scored against my own re-run, never the report.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the tail escape is rejected | `./target/release/wat --check docs/excursus/2026/08/002-handle-lifetime-wall/probes/red-tail-escape.wat` | REJECTED, naming the escaping function and the service |
| 2 | the binding form beside it compiles | same file, same run | the `held`-shaped twin is NOT named. If it is, condition 2 was ignored and every non-tail let is a false positive |
| 3 | a BUILTIN tail head does not fire | a let creating a handle whose tail form is `(:wat::i64::+ … 0)` carrying the peer | NOT rejected — a builtin head emits no `TailCall`, so the scope survives. This is the `D-body-nontail` case, measured green at runtime |
| 4 | stone 1 still holds | `cargo nextest run --release -E 'test(probe_ex002_creation_escape)'` | 1 passed — stone 2 must not disturb stone 1's rule |
| 5 | the census, RUN not grepped | build, then `--check` every corpus `.wat` | rejects ONLY: `red-tail-escape.wat`, `probe-self-sched-bisect.wat` ×3, `probe-tail-scope-sees-bindings.wat` ×1. **Anything else is a finding to report, not to rune** |
| 6 | the bisect probe still RUNS | `./target/release/wat wat-scripts/scratch-pad/probe-self-sched-bisect.wat` | prints its table. A moved-or-rejected instrument is a FAIL: the discrimination it prints is this excursus's evidence |
| 7 | ★ ONE tail-form list, or a drift gate | `git diff` — find where the seven forms are enumerated | either ONE shared constant read by `eval_tail`'s dispatch and the checker, or a test that fails when two lists disagree. **A duplicated list with no gate is a FAIL even with a green floor** — that is the failure mode nothing goes red for |
| 8 | tail semantics unchanged | `git diff src/runtime.rs` | list may MOVE; `eval_tail` must dispatch on the same seven forms after. No behavioural change |
| 9 | runes are on instruments only | `git diff` for `rune:` | runes on `probe-self-sched-bisect.wat` (and `probe-tail-scope-sees-bindings.wat` if needed). **A rune on `red-tail-escape.wat` is an automatic FAIL** — it silences the wall's only proof it fires |
| 10 | the floor | `./scripts/floor.sh` — read the Summary line, never a piped exit code | 5133+ run, 0 failed, FLOOR=0. A red is a red: do not re-run, name the arm |
| 11 | the error teaches | read the rejection text | names the service, the creating span, and the tail-call span — the three facts whose absence cost 38 days |
| 12 | the self-scheduling fixture is untouched | `git diff tests/services/probe_arc278_self_scheduling.wat` | empty. Its drive already sits in a binding; if the wall forces a change there, the rule is wrong |

**Runtime prediction:** 60–120 minutes. Most of it is threading tail position and settling row 7;
the rule itself reuses stone 1 wholesale.

## Trap doors, named in advance

- **Ignoring condition 2** (the let must itself be in tail position). Cheap to skip, and it turns
  every non-tail let into a false positive. Rows 2 and 3 exist for this.
- **A second tail-form list.** The seductive one: it works today, passes every row, and rots
  silently the first time a form gains a tail variant. Row 7 is the only thing standing there.
- **Runing the acceptance criterion** to get a green floor — refused correctly on stone 1, and row 9
  makes it an automatic fail here.
- **Firing on nothing.** Rows 2, 3, 4, 8, 10, 12 all pass for a wall that never fires. Only rows 1
  and 5 catch it. A green floor with no rejections is a FAILURE.
