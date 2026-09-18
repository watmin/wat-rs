# REVIEW — orchestrator's independent re-run of the mode-parity gate

> Weighed against my own re-run, not the SCORE. Floor run whole; source read line by line.

## What HOLDS — verified by me, not credited to the report

| # | row | my verification |
|---|---|---|
| 1,2 ★ | both arms RED at HEAD **for the right reason** | floor log: `mode_parity.rs:144` → `SOUNDNESS: --check Accepted but run failed before :user::main (check=Accepted run=Rejected)`; `mode_parity.rs:168` → `LIVENESS: run terminated normally but --check died by signal (check=DiedBySignal(6) run=Accepted)` |
| 3 ★ | control discriminates | `mode_parity_good`, `mode_parity_calibration` pass; `wat_cli__check_bad.wat` exists and is a real fixture |
| 5 | no exit-code table, no depth constant | `Outcome{Accepted,Rejected,DiedBySignal(i32)}` via `ExitStatus::code()`; depth lives in the generator |
| 6 | blast radius | `git diff --name-only a226ded45..HEAD | grep '^src/'` → **empty**. 6 new files, all `tests/cli/` |
| **7** | **floor — the row the SCORE left unverified** | **I ran it: `Summary [445.735s] 5425 tests run: 5423 passed (1 slow), 2 failed, 21 skipped`.** The 2 are exactly the arms; `no_inlined_wat_in_tests` is GREEN — the fixture move held. 5420 + 5 new = 5425. **Nothing else moved.** |

The generator, the subprocess-only discipline, `CARGO_BIN_EXE_wat`, the tail-vs-non-tail finding
(tail chain at 1000 does **not** redden — a real fact about the engine, honestly reported), and the
capture-don't-re-run on the unexpected `no_inlined_wat_in_tests` red: all correct.

## ⛔ REJECTED — the gate cannot survive its own cure

**Both arms contain contradictory assertion pairs on the same value in the same function.**

```
mode_parity.rs:139-140   assert_eq!(run, Outcome::Rejected)          requires Rejected
mode_parity.rs:145       assert!(run == Outcome::Accepted)           requires Accepted

mode_parity.rs:159       assert!( matches!(check, DiedBySignal(_)))  requires signal death
mode_parity.rs:169       assert!(!matches!(check, DiedBySignal(_)))  requires NOT signal death
```

No behaviour of any engine satisfies either pair. These tests are **RED unconditionally** — not
"RED at HEAD". They are `assert!(false)` with provenance. The red they emit today carries no
information, because they would emit red on a fully cured engine too.

**A gate that cannot go green cannot certify a cure**, and this strike exists precisely because the
arc shipped two gates that could not go red (`right_index_counter_invariant.rs`, and the beta
census `experiri` mutation-proved blind). Swapping one failure mode for its mirror is not progress.

### And the SOUNDNESS consequent is not the invariant

`:145` asserts **`run == Accepted`** unconditionally. The invariant is an *implication*:

> `--check` Accepted ⟹ the run path does not fail before `:user::main`

The empty fixture legitimately has **no `:user::main`**, so `run` is `Rejected` at HEAD and will
remain `Rejected` after any correct cure. As written, the arm demands that running an empty program
SUCCEED. That is not the contract; it drops the antecedent and mis-states the consequent.

### The correct shape is 40 lines away, in this same file

`mode_parity_good:186-196` already writes it properly:

```rust
if check == Outcome::Accepted {
    assert_eq!(run, Outcome::Accepted, "SOUNDNESS on control: ...");
}
```

**The control encodes the implication; the two arms drop the antecedent.** One file, both shapes.

## Also — the non-vacuity guard is itself vacuous

`mode_parity_cases_are_named:78` — `assert!(!cases.is_empty(), "…the gate is measuring nothing")`
where `cases` is a **fixed-size array literal**. `!cases.is_empty()` is a compile-time `true`; the
assertion has one possible outcome. The idiom the BRIEF pointed at
(`docs_wat_loads_or_declares_why_not.rs:97-104`) asserts non-empty on a list built by **walking the
filesystem**, which genuinely can come back empty — that is what makes it a guard.

The `p.is_file()` loop beside it **is** real and is the part doing the work. Keep it; drop or
re-aim the tautology. The live risk here is not "the list is empty" (impossible) but "an arm stopped
being covered" — which nothing currently checks.

## What to change — three edits, no rewrite

1. **Delete the defect-pinning `assert_eq!`s** at `:135`, `:139-140`, `:159`. Make them `eprintln!`
   if the observed HEAD values are worth recording in the log; they are documentation, not contract.
2. **State each arm as the implication**, using the control's own shape:
   - SOUNDNESS: `if check == Accepted { assert_ne!(run, Rejected, "…") }`
   - LIVENESS:  `if run != DiedBySignal(_) && run == Accepted { assert!(!matches!(check, DiedBySignal(_)), "…") }`
   Then both arms are RED at HEAD **and GREEN when cured**, which is what a gate is.
3. **Re-aim the non-vacuity assertion** at something that can fail — e.g. assert the case list
   covers both arms by name, or drop it and keep `is_file()`.

**Then mutation-prove it**: with the arms restated, a stub that forces `check` to the cured value
must turn each arm GREEN, and only that arm. If an arm stays red under its own cure, it is still a
monument.

## Verdict

**Instrument accepted in substance, rejected in form.** The fixtures, the generator, the
classification, the calibration, the blast radius and the floor are all sound and I verified each.
The three assertion defects are ~15 lines and do not touch any of that.
