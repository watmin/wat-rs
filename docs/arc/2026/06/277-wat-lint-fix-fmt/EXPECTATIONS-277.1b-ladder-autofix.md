# EXPECTATIONS — Arc 277.1b (weigh on the orchestrator's own build)

| what | command | expected |
|---|---|---|
| ladder-autofix gate | `cargo test --release -p wat --test probe_arc277_1b_ladder_autofix` | 1 passed / 0 failed (rewrite → `contains?`/`HashSet`, nested-if gone) |
| ladder rule reports | `cargo test --release -p wat --test probe_arc277_lint_if_ladder` | 1 passed / 0 failed |
| concat rule intact | `cargo test --release -p wat --test probe_arc277_lint_concat_abuse` | 1 passed / 0 failed |
| deftest binary | `cargo test --release --test test 2>&1 \| grep "test result"` | 260 passed / 1 failed (was 259; +1 Case 7) |
| deporder gate | `cargo test --release --test test_stdlib_load_order 2>&1 \| grep result` | 1 passed / 0 failed |
| lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 929 passed / 36 failed (UNCHANGED) |

Runtime prediction: 18–28 min (one record + a field-type change + an applier; rides existing fix.wat).

## Trap-doors named

- **`Option<FixEdit>` ripple** — all 3 Finding constructors must pass a valid `fix` (None for two,
  Some for the ladder). A missed constructor = uncompilable (the typed-record win) — good, not a trap.
- **Offset math off-by-one** — `old-len = offset-of(end) − offset-of(start)` must equal the form's exact
  char length. `ast-end-span` is one-past-the-`)`, so this is exact. The probe's "nested-if gone" +
  "contains? present" catches a wrong extent (it'd leave a dangling `)` or eat a neighbor).
- **Right-to-left apply** — edits must be applied descending-offset (`reverse` the ascending findings),
  or earlier splices shift later offsets. For the single-ladder fixture it's one edit, but the applier
  must be correct for N.
- **new-text via format** — dogfooding `format` to build the contains? text; if the template's literal
  braces/quotes trip something, STOP. (It shouldn't — no `{{`, the `{lits}`/`{var}` are placeholders.)
- **deporder** — lint.wat now calls fix.wat fns (first cross-dep). fix.wat precedes lint.wat in
  stdlib.rs, so 0 violations expected; a non-zero gate is a real load-order find (STOP-2).

## Definition of done

All six rows match. `Finding.fix` is `Option<FixEdit>`; the ladder rule emits `Some`, the other two
emit `None`; `lint-fix-file` rewrites a ladder to `contains?` and round-trips a clean file byte-identical.
`wat/lint.wat` + `wat/fix.wat` + `wat-tests/lint.wat` + the probe are the only changes. The
concat→format auto-fix is the NEXT stone, named not built (exigere).
