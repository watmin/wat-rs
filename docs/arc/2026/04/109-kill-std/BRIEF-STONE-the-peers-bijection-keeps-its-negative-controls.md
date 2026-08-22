# BRIEF — the `:peers` bijection keeps its negative controls

This stone ships the artifact the previous stone's brief **told the rider to delete**. It adds no
capability. It makes permanent five checks that currently exist only in a scoring write-up, where
nothing re-runs them.

## Why this exists — read this part, it is the whole point

`BRIEF-STONE-defservice-compares-types-as-data.md` named its row 4 the load-bearing one:

> *"Row 4 is the one that bites. Rows 1-3 measure that the checks still accept; only row 4 measures
> that they still refuse. A rewrite that returns 'equal' unconditionally passes 1, 2, 3 and 6."*

…and then, three sections later, instructed: *"Delete any scratch `.wat` that must fail."* The rider
obeyed both. The rewrite is correct — I rebuilt the controls by hand and every one passes, recorded
verbatim in `SCORE-STONE-defservice-compares-types-as-data.md` — but **the only evidence that the
bijection still rejects anything now lives in prose**, and prose does not go red.

The deletion instruction had a true premise (`tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`, so a must-fail file there is a red floor) and a false conclusion
(therefore it cannot be kept). **`tests/**` is not covered by that gate** — which is exactly why the
must-fail fixtures below already exist in the tree.

## Read in order

1. `tests/macros/probe_arc279_format.rs` — **the exemplar. Copy its shape.** Its two
   `*_is_macro_error` tests are precisely this stone's shape: `startup_from_file(<a fixture that must
   fail>)` → assert `is_err()` → `wat::assert_edn_matches_file!` against a golden `.edn` sibling.
2. `tests/macros/probe_arc279_format_missing_kwarg.wat` — a must-fail fixture living under `tests/`.
3. `tests/services/probe_arc278_s2s_peer_on_thread.wat` — **the base fixture.** Every case below is
   this file with ONE clause changed. Its `:ephemeral` peer field is at :48, its `:peers` at :50.
4. `tests/services/mod.rs` — three lines; it says *"Add a test: drop a .rs here."* `build.rs`
   generates the module list. **No registration step; do not add one.**
5. `wat/service.wat:880-913` — the two bijection checks these cases exercise, "missing" then "extra".

## The work

Add one `.rs` under `tests/services/` plus its fixtures and goldens. Five cases, each the base
fixture differing in one clause. The first two exist to prove the *old* spelling still rejects; the
last three are the new spelling, and **case 5 is the one that carries the stone**.

| case | ephemeral peer written as | `:peers` | must |
|---|---|---|---|
| 1 | `Peer<probe::Echo::Op,probe::Echo::Reply>` | `[:probe::Bogus]` | FAIL — "missing" |
| 2 | `Peer<probe::Echo::Op,probe::Echo::Reply>` | *clause deleted* | FAIL — "extra" |
| 3 | `(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])` | `[:probe::Echo]` | **PASS** |
| 4 | `(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])` | `[:probe::Bogus]` | FAIL — "missing" |
| 5 | `(:wat::kernel::Peer :- [:probe::Echo::Op :probe::Echo::Reply])` | *clause deleted* | FAIL — "extra", **and the message must contain `probe::Echo`** |

★ **Case 5 is the non-vacuity control and the reason the other four are not enough.** The bijection
compares two lists of surface names. If the structural reader that extracts a surface from a
form-spelled `Peer` silently returned *nothing*, cases 1, 2 and 4 would still fail with the same
messages — they fail on the `:peers` side. Only case 5 fails on the *ephemeral* side, so only case 5
can tell "the reader works" from "the reader returns an empty list." Its assertion must check that
the diagnostic **names `probe::Echo`**, not merely that startup errored.

Case 3 is the positive control; without it the four failures prove nothing about the accept path.

## Verified destinations — these all ran, exactly as written, before this brief

I built all five by `sed` off the base fixture and ran `target/release/wat --check` on each:

```
baseline (unmodified)   exit 0
case 1                  exit 1   ":peers declares surface :probe::Bogus but no :ephemeral field is typed …"
case 2                  exit 1   ":ephemeral holds a dialed Peer<probe::Echo::Op,…::Reply> but surface
                                  :probe::Echo is not declared in :peers …"
case 3                  exit 0
case 4                  exit 1   the "missing" message, same as case 1
case 5                  exit 1   the "extra" message, naming :probe::Echo
```

Note the driver: the exemplar uses `startup_from_file`, not `--check`. Both reach the macro
expansion; use `startup_from_file` to match the exemplar and to get the `Err` value the golden
compares against.

## Goldens

Write the assertions first and let the harness capture the goldens:
`UPDATE_EDN=1 cargo nextest run --release -p wat --test services -E '…'` writes each `.edn`
(`src/lib.rs:248`). Then **read every captured golden** and confirm it holds the message you expected
— a captured golden records whatever happened, including a wrong thing. Then re-run WITHOUT
`UPDATE_EDN` and confirm green.

⚠ `UPDATE_EDN=1` rewrites **every** golden the selected tests touch, including ones that were already
passing, re-pretty-printing them. Scope the `-E` filter to your new tests, and if `git status` shows a
golden you did not add, revert that file.

## What "done" looks like

1. Five cases, five named tests, each asserting the outcome in the table.
2. Case 5 asserts the diagnostic **contains `probe::Echo`**.
3. `cargo nextest run --release -E 'binary_id(wat::services)'` green.
4. Perturb-to-prove: temporarily make one bijection check unconditionally succeed in
   `wat/service.wat`, confirm your new tests go **RED**, then revert that edit and confirm green
   again. Report both numbers. **A test suite that cannot go red for the defect it names is
   decoration.** Revert the perturbation — leave `wat/service.wat` byte-identical to how you found it
   (`git diff wat/service.wat` must be empty when you finish).

## Boundaries

- New files under `tests/services/` only, plus the goldens. **`wat/service.wat` must end byte-identical.**
- Do NOT run `scripts/floor.sh` or a full `cargo nextest`. The orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT put any must-fail `.wat` under `wat-scripts/` — that directory IS loader-gated and a
  must-fail file there is a red floor. `tests/` is the correct home; that is this stone's thesis.

## STOP triggers — ship nothing further and report

- **STOP-1.** If case 3 does not pass, the form spelling is not accepted where I measured that it is.
  STOP — that is a substrate finding, not a test to adjust.
- **STOP-2.** If step 4's perturbation does NOT turn your tests red, they are not measuring the
  bijection. STOP and report what they are measuring instead.
- **STOP-3.** If a captured golden holds a message different from the table above, STOP and report
  the difference verbatim. Do not adjust the expectation to match what you got.

## Your report

The five test names and what each asserts. The perturb-to-prove numbers, both directions. Every
golden's content. Anything about the exemplar's shape that did not transfer, and what you did instead.
