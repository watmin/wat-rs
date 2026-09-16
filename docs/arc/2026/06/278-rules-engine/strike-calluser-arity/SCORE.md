# SCORE — D3, weighed against the orchestrator's own re-run

> Re-run here at `057f9d494`. The rider's report is cited only where it reports something I cannot
> reconstruct.

## The scorecard, re-run

| # | expected | actual |
|---|---|---|
| 1 | control green before | ✅ (see the note on row 1 below — as written it could only be scored as the two controls passing pre-fix) |
| 2 | untampered answers 1 | ✅ — **and this control turned out to be insufficient; see the vacuity mutation** |
| 3 | arm 1 RED by a wrong answer | ✅ accepted, hits **0** |
| 4 | arm 2 RED by a dropped arg | ✅ accepted, hits **2** |
| 5 | arm 3 RED, not merely "an error" | ✅ `UnboundSymbol("slot 1")`, not an `ArityMismatch` |
| 6 | all GREEN after | ✅ — **five** arms, not three |
| 7 | the surplus branch gone | ✅ `grep 'else if i < inner.len()'` → no hit; the loop is `program.params[i]`, total by construction |
| 8 | no second copy of the invariant | ✅ `src/rete/export.rs` is not in the diff (my own grep for this row matched `probe_arc278_export.rs` as a substring — the instrument was wrong, not the work) |
| 9 | blast radius | ✅ `expr_ir/eval.rs` + `probe_arc278_export.rs` |
| 10 | floor | ✅ `Summary [ 404.478s] 5202 tests run: 5202 passed (1 slow), 21 skipped`, **zero FAIL rows** — after curing a red, below |
| 11 | clippy | ✅ rc=0, zero warnings |

## ⛔ THE FLOOR WENT RED, AND THE RIDER'S SCOPED CHECK COULD NOT HAVE SEEN IT

First floor: `5202 run, 1 failed`. Captured whole, not re-run. The exact arm:

```
FAIL  wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
      panicked at tests/lint/no_loose_string_assert.rs:122:5
      🔥 LOOSE STRING ASSERTIONS — 1 site(s) …
      Offenders:
      tests/rete/probe_arc278_export.rs:959
```

The new probe asserted `op.contains("call-user")` where `CALL_USER_OP` is a fixed constant
(`eval.rs:381`). Cured at the root — an exact `assert_eq!`, **not** a `rune:lint(loose-assert)`
exemption; the value is deterministic and known, so the lint is simply right.

**The gap is mine.** The rider ran `binary_id(wat::rete)` (434/434) because that is the binary its
change lives in. New *test code* trips lints that live in `wat::lint`, and my brief never named it
as part of the rider's scoped checks. **Any brief that adds tests must name `binary_id(wat::lint)`
alongside the subject binary**, or the lint red lands on the orchestrator's floor every time.

## The mutation I re-drove myself — and it corrected my own control

The rider drove restore-the-branch (stays green). I drove the one that tests for **vacuity**:
force the check to refuse *every* call.

```
FAIL  a_well_formed_user_call_still_runs                 ← the rider's own added control
FAIL  probe_arc278_reduce_arity_totality::the_total_three_arity_form_still_fires
PASS  untampered_export_answers_one_hit                  ← MY row 2 control
```

**My row 2 control cannot see a refuse-everything check**, because the untampered fixture's fence
never reaches `exec_program_on`. Every green in this strike would have been consistent with "the
check refuses everything" if the suite had contained only what I specified. The control that
carries the weight is one the rider added on its own initiative.

## ⛔ Where MY brief was thin — four, and one is a phantom

- **A. ★ DESIGN's ⚠ named a function that does not exist.** I cited `callee_program` as the HOF
  path that extracts a program without running it. `grep -rn 'fn callee_program' src/rete/` returns
  **nothing**; it is `compiled_fn_arg` (`eval.rs:488`). The *reasoning* was right and the rider
  re-derived it independently — but a rider checking my ⚠ by grepping the symbol I named finds an
  empty result, and an empty result is not a disproof of anything. I asserted a symbol I had not
  grepped.
- **B. ★ I named ONE internal call site; there are SIX.** Read-item 4 said `exec_foldl`.
  `grep -n 'exec_program_on('` → `:12` (`exec_call`, driven from `fire/acc.rs:484`), `:366`, `:372`,
  `:482` (foldl), `:575` (reduce), `:610` (mapv), `:642` (filterv). The check newly enforces
  "exactly 2 params" on foldl/reduce and "exactly 1" on mapv/filterv/acc-fold. It holds — but
  **this is the second consecutive strike where I named one site and the real set was several**
  (A6: one tower, three). Promoted to memory.
- **C. The `args.is_empty()` exemption is a real semantic decision, not a no-op.** `lower_fn`
  (`mod.rs:899`) compiles **every** literal `fn` — not only in HOF position — to
  `CallUser { program with params, args: [] }`. So `args.is_empty()` with non-empty `params` also
  means *"a fn **value** reached exec"*, which now answers `ArityMismatch { expected: n, got: 0 }`
  where it previously answered `UnboundSymbol`. Both are errors, so nothing regressed, and a fn
  value is not a call — but this belonged in the DESIGN, not in a future bisect. **Recorded as an
  accepted consequence.** The short-circuit's comment (*"Literal fn value — foldl applies it via
  exec_foldl"*) is now the misleading half; left in place as out-of-radius, and it is a small
  instance of the same alibi shape A6 turned up.
- **D. Row 1 as written cannot be run.** "`binary_id(wat::rete)` green before" is only true before
  the probes exist, since they are RED by design. Score it as the controls passing pre-fix, and
  write the row that way next time.

## Arms not driven, named

None. Five arms and two controls, all driven. Two of the five were the rider's own additions and
both earn their place: **arm 3-mine** (arguments to a *zero-parameter* callee — the only case where
the deleted branch was the sole writer, fabricating `inner[0]` for a slot never declared a
parameter) and **arm 5** (too few but non-zero, which reaches the check by the *evaluating* path
rather than the `args.is_empty()` short-circuit my arm 3 rode).
