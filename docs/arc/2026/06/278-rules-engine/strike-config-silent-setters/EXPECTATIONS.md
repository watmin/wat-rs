# EXPECTATIONS — Ω4: silent config setters

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | Ω4a cured | typo'd setter → **located error, non-zero exit**; never a silent default |
| 2 ★ | Ω4b cured | valid setter after a body form → `SetterAfterNonSetter`, **located**, non-zero exit |
| 3 ★ | the variant is REACHABLE | `SetterAfterNonSetter` is constructed on a real path — grep it, and the gate drives it |
| 4 ★ | accessors still legal | `(:wat::config::dim-count)` in `:user::main` still works; control prints `4096` |
| 5 | one name grammar | no hand-rolled `rsplit("::")`; `tests/lint/one_name_grammar.rs` green |
| 6 | floor | `./scripts/floor.sh` → **0 failed** |
| 7 | clippy | rc=0 |
| 8 | blast radius | `src/config.rs` + tests; **zero lines in `resolve/` or `check.rs`** |

★ load-bearing. **Row 6 is the deliverable.**

## Runtime prediction

30–50 min. The scan is ~15 lines; the fixtures are four small `.wat` files.

## Trap doors, named in advance

- **Outlawing accessors.** The cure must not reject `(:wat::config::dim-count)` in the body. Row 4
  exists for this and it is the control — if it reddens, the discriminator is wrong.
- **A second name parser.** `config.rs:457-461` says a hand-rolled `rsplit` here was already caught
  on the floor once. STOP-2.
- **The corpus may already contain an offending form.** STOP-1. If it does, that is a finding.
- **`RequiredFieldMissing` is dead too and is NOT in scope.** Making it reachable changes whether an
  empty entry file is legal — a user-facing contract change. STOP-3.
- **Re-run the floor at FINAL state.** Two gates fired unexpectedly in the mode-parity strike, and
  the second was caught only by re-running after the last edit.

## What would make me reject the result

- A control that reddens (accessors outlawed).
- `SetterAfterNonSetter` still unconstructed on any real path.
- A diagnostic with no span.
- Any change under `resolve/` or `check.rs`.
- A red floor of any size.
