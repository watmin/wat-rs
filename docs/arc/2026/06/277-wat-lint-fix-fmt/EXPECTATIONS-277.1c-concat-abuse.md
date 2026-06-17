# EXPECTATIONS — Arc 277.1c (weigh on the orchestrator's own build)

| what | command | expected |
|---|---|---|
| concat-abuse gate | `cargo test --release -p wat --test probe_arc277_lint_concat_abuse` | 1 passed / 0 failed / 0 ignored |
| ladder rule intact | `cargo test --release -p wat --test probe_arc277_lint_if_ladder` | 1 passed / 0 failed |
| deftest binary | `cargo test --release --test test 2>&1 \| grep "test result"` | 259 passed / 1 failed (was 257; +2 lint deftests; the 1 = run_string_entry_direct) |
| deporder gate | `cargo test --release --test test_stdlib_load_order 2>&1 \| grep result` | 1 passed / 0 failed |
| lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 929 passed / 36 failed (pre-existing) |

Runtime prediction: 12–20 min (one rule copied from a worked template).

## Trap-doors named

- **Fold order regression** — the new rule wired BEFORE the ladder rule would make Case 1's "first
  finding == nested-if-=-ladder" fail. Must be AFTER. (STOP-2.)
- **lint-stdlib now noisy** — `lint-stdlib` (Case 4) will start surfacing many concat-abuse findings
  (the stdlib has real ones: `violation->finding`, `make-ladder-finding`, format's macro-error concats).
  Case 4 asserts `length >= 0`, so it still passes — but confirm it does, and confirm it's not throwing.
- **`Tuple(i64,i64)` at runtime** — proven at macro-eval; should be fine at runtime, but STOP-1 covers it.
- **Deftest count** — exactly +2 (Cases 5 & 6). If the binary reports anything other than 259/1,
  read why before claiming green.

## Definition of done

All five rows match. The rule is report-only (`fix ""`, severity `"warn"`, rule `"concat-abuse"`). The
two deftests + the un-ignored probe are green. `wat/lint.wat` + `wat-tests/lint.wat` + the probe are the
only changes. No deferral language in the rule (exigere) — the auto-fix's absence is named to its
keystone (277.1b / ast-end-span) in the DESIGN, not hand-waved in the code.
