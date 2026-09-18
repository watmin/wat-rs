# REVIEW 2 — independent re-run of the restated mode-parity gate

> Weighed against my own drive and my own floor, not the SCORE.

## What the restatement FIXED — verified, and it is real

All three defects from REVIEW are gone:
- both arms are implications (`if antecedent { assert }`), not contradictory pairs;
- HEAD observations are `eprintln!`, not contract;
- the non-vacuity guard no longer asserts `!is_empty()` on an array literal.

At HEAD both arms are RED for the right reason:
```
mode_parity.rs  SOUNDNESS: --check Accepted but run Rejected before :user::main (check=Accepted run=Rejected)
mode_parity.rs  LIVENESS:  run terminated normally but --check died by signal (check=DiedBySignal(6) run=Accepted)
```

## ⛔ 1 — THE MUTATION PROOF IS AIMED PAST ITS SUBJECT. Driven.

`soundness_holds` (`:71`) and `liveness_holds` (`:80`) are called from **exactly four sites**:
`:220`, `:224`, `:234`, `:238` — all four inside the two mutation tests. **Neither arm calls
either function.** `mode_parity_empty:164` and `mode_parity_deep_freeze_recursion:182` inline
equivalent logic.

So the mutation tests prove the *predicates* are correct. They prove nothing about the *arms*.

**Driven, not argued.** One token in the SOUNDNESS arm — `check == Outcome::Accepted` →
`check != Outcome::Accepted` — then `cargo nextest run --release -E 'test(mode_parity)'`:

```
Summary [1.263s] 7 tests run: 6 passed, 1 failed
  PASS  mode_parity_empty                                 <- the LIVE defect, now invisible
  PASS  mode_parity_soundness_cure_greens_only_that_arm   <- did not notice
  PASS  mode_parity_liveness_cure_greens_only_that_arm    <- did not notice
  PASS  mode_parity_cases_are_named
  PASS  mode_parity_good
  PASS  mode_parity_calibration
  FAIL  mode_parity_deep_freeze_recursion                 <- the other, untouched arm
```

A one-token change silently disabled the SOUNDNESS arm and **both mutation tests stayed green.**
(Mutation reverted; file byte-identical to grok's, verified by `diff`.)

**This is CLASS A from our own work list** — one rule encoded twice, held apart, nothing forcing
agreement — landing on the proof itself. `[[mutation-prove-every-gate]]` is unpaid until the stub
and the arm run the same code.

**Fix, one line per arm:**
```rust
assert!(soundness_holds(check, run), "SOUNDNESS: ... (check={check:?} run={run:?})");
assert!(liveness_holds(check, run),  "LIVENESS: ... (check={check:?} run={run:?})");
```
Then blinding either the predicate or the arm reddens both, and the stub is a real proof.

## ⛔ 2 — THE FLOOR IS RED ON A THIRD TEST, AND THE SCORE DID NOT SEE IT

The SCORE says *"Floor first-capture and clippy as previously scored; not re-run."* I ran it:

```
Summary [445.558s] 5427 tests run: 5424 passed (1 slow), 3 failed, 21 skipped
  FAIL  wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  FAIL  wat::cli  mode_parity::mode_parity_empty                    (deliverable)
  FAIL  wat::cli  mode_parity::mode_parity_deep_freeze_recursion    (deliverable)
```

Verbatim, `tests/lint/no_loose_string_assert.rs:135`:

> 🔥🔥🔥 LOOSE STRING ASSERTIONS — 3 site(s) assert a value with contains/starts_with/
> ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
> malformed maps, and appended garbage.
> …
> Offenders:
> tests/cli/mode_parity.rs:108
> tests/cli/mode_parity.rs:112
> tests/cli/mode_parity.rs:116

Those three are the `cases.iter().any(|c| c.name.contains("empty" | "deep" | "good"))` lines.

**My REVIEW steered you into this** — it said *"assert the case list covers both arms by name"* and
`.contains` is the loose reading of that. The gate is right and my wording was not precise. The
exact-comparison form is also strictly stronger:

```rust
let names: Vec<&str> = cases.iter().map(|c| c.name).collect();
assert_eq!(
    names,
    ["mode_parity__empty.wat", "mode_parity__deep_freeze_recursion.wat", "mode_parity__good.wat"],
    "an arm's fixture was dropped or renamed — the gate is not covering that arm"
);
```
One exact assertion; reddens on a drop, a rename, or a reorder. Keep the `p.is_file()` loop.

## Note for the record — this strike has now tripped TWO gates neither of us anticipated

`no_inlined_wat_in_tests` (round 1, caught by grok's floor, fixed) and `no_loose_string_assert`
(round 2, caught only because I re-ran). **The floor must be re-run at final state, every round.**
A first-capture from before the last edit is a photograph, not a verdict.

## Verdict

Two edits: the arms call the predicates; the non-vacuity check becomes one `assert_eq!`. Then
**re-run the whole floor** and expect exactly two failures — `mode_parity_empty` and
`mode_parity_deep_freeze_recursion` — and nothing else.
