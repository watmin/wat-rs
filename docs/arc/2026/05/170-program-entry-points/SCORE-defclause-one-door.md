# SCORE — one door for defclause registration (IN PROGRESS)

Brief: `BRIEF-defclause-metadata-reaches-stdlib.md`. Baseline `c7368bbc`, floor
`4105 tests run: 4105 passed, 303 skipped` (orchestrator's own `--release` run).

## Verified by the orchestrator's own hand

- **The shape.** `register_defclause(form, privilege, phase, sym)` +
  `ClauseRegPhase::{Stub, Runtime}` (`src/runtime.rs:745`, `:798`). All four former
  sites are four calls and none moved, so the freeze ordering is untouched:
  `:1013` Stub (user pre-reg) · `:1253` Stub (stdlib stub) · `:1166` Runtime (stdlib) ·
  `:2357` Runtime (eval-time / freeze step 9).
- **Effect 3 is unconditional** on privilege and phase (`:815-819`) — a defclause is a
  defclause; where its form was loaded from is not a property its metadata knows about.
- **The reserved-prefix distinction survived** the collapse rather than being flattened
  (`:827-828`): stdlib bypasses `is_reserved_prefix`, user does not.
- **STOP-4 not tripped.** `src/check.rs`'s diff is entirely inside `mod tests`; the
  `walk_for_restricted_call` / `extract_prefix_list_from_metadata` walker is untouched.
- **The RED half of the acceptance condition.** With Effect 3 removed, ONLY
  `defclause_metadata_gap_stdlib_registered_restricted_to_enforced` fails — "a
  stdlib-registered defclause's `{:restricted-to […]}` must be enforced against a caller
  outside the whitelist" — while the two malformed-form gates stay green, correctly,
  since they do not depend on the insert. **The gate dies with the mechanism.** Restored,
  with a permanent note at the insert recording that it is load-bearing and proven both
  ways.

- **The GREEN floor**, on a quiescent tree the orchestrator owned alone, three times:

  ```
  RUN1 | 4108 tests run: 4108 passed, 303 skipped
  RUN2 | 4108 tests run: 4108 passed, 303 skipped
  RUN3 | 4108 tests run: 4108 passed, 303 skipped
  ```

  `4108 = 4105` baseline `+ 3` new gates. Zero failures in all three.

- **The rider's RED, independently reproduced.** Two hands, separately, removed Effect 3
  and got the same discrimination: only
  `defclause_metadata_gap_stdlib_registered_restricted_to_enforced` falls, and the two
  malformed-form gates hold. The gate is coupled to the mechanism.

## REJECTED disposition — the flake claim

The rider proposed, of `deftest_wat_tests_service_request_malformed_on_process`:

> *"an unrelated process-spawn IPC test … that passed cleanly in isolation — a timing
> flake under full-suite parallel load, not a regression from this change."*

Rejected on three counts:

1. **"passed cleanly in isolation" is not a disposition.** It describes the search, not
   the defect. A real race introduced by a change also passes in isolation — that is the
   definition of the thing, not evidence against it.
2. **"unrelated" is false on its face.** `spawn-program` **is a defclause**
   (`wat/spawn.wat:262`). This stone changes how defclauses register and which phase
   writes what into `sym`. A process-spawn IPC test is not adjacent to that blast radius;
   it is inside it.
3. **"a timing flake under parallel load" is an asserted mechanism with no evidence.** A
   mechanism must be proven. This one was reached for because it is the dismissal that
   costs nothing.

It also lands one stone after R59 recorded the identical move, and against the standing
ruling: *there are no preexisting failures — zero for a week.*

## The measurement, run — and the honest disposition

| condition | result |
|---|---|
| isolated, `--test-threads=1`, N=20, **with** the change | **20 pass / 0 fail** |
| full-suite parallel load, **with** the change, ×3 | **3 × `4108/4108/0`** |

The stashed side was **not** run: with zero failures on the with-change side there is
nothing to attribute, so the comparison would have been theatre rather than evidence.

**Disposition: NOT REPRODUCED — deliberately not closed as "flake."** "Not reproducible"
describes the search, not the defect
([[feedback_not_reproducible_is_not_a_disposition]]).

What raises this above a shrug is a mechanism with evidence: **every observed failure
occurred inside the window where two `nextest` runs were provably concurrent** —
`ps`-confirmed (the rider's PID 1017542 alongside the orchestrator's own run), each
process building and writing the same tree and contending on one `target/` lock. In that
window the orchestrator's run showed THREE failures (`HolographicLru`,
`core_equality_typed_i64_eq`, `edn write_json_string`) and the rider's interim showed ONE
(`service_request_malformed_on_process`) — four different tests, no overlap, all
process/thread-harness shaped. Since quiescence: zero failures across three solo
full-load runs and twenty isolated ones.

That is a strong correlation with a known mechanism, not a proof. It stays an **open
observation** attached to this score. If `service_request_malformed_on_process` reddens
again on a quiescent tree, this note is the prior — and the next hand should suspect the
harness, not re-litigate this stone.

**The rider's reasoning is rejected regardless of where its conclusion lands.** "Passed
cleanly in isolation → a timing flake, not a regression" is invalid: a load-dependent race
introduced by a change passes serially by construction. Being right by luck is not being
right, and `spawn-program` IS a defclause — a process-spawn IPC test was the least
defensible thing in the suite to call "unrelated" to a stone that rewrites defclause
registration.

## The disposition protocol that was followed — a DIFFERENTIAL, not a re-run

Run only on a quiescent tree the orchestrator owns alone (no live rider; verify with
`ps -eo pid,etimes,args | grep -E "[c]argo|[n]extest"`).

1. `deftest_wat_tests_service_request_malformed_on_process`, `--test-threads=1`, N≥20,
   **with** the change. Any red = real.
2. The same test N≥20 **under full-suite parallel load**, with the change.
3. `git stash` the change; repeat 1 and 2.
4. **The differential is the disposition.** Fails only with the change on the tree → it is
   a regression and the flake story is dead. Fails identically on both sides → only then
   is "pre-existing" on the table, and it still owes a mechanism.

Anything short of this closes a flake on a green run — the move that already cost a week
of a dead protocol (R59 `NISI FRANGAS, NIHIL PROBAS`).

## Open

- The differential above.
- The rider's answers to the brief's four report questions (shape, out-of-file changes,
  whether it ever held a clean green and the verbatim Summary, honest deltas).
- Whether `:1253`'s `let _ = register_defclause(…)` (a swallowed `Result`, unchanged from
  the prior `if let Ok((name, _cs))`) belongs in the no-hidden-failures sweep. Not a
  regression; the malformed-form error still surfaces at the definition via the
  `Runtime`-phase call's `?`, and the two malformed-form gates prove it.
