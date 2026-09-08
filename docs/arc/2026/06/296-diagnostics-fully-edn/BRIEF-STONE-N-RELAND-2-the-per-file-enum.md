# BRIEF — STONE N RELAND 2: the per-file enum the codemod cannot see

> RELAND 1 landed the hard part: Path B's generated `::Op::` send, in `src/runtime.rs`. `println`
> runs, `readln` runs, and the timeouts are gone. This closes what is left.

## WHERE IT STANDS — measured centrally

```
M            2447 passed · 2773 failed · 18 TIMED OUT
N            4765 passed ·  473 failed ·  0 timed out
N RELAND-1   4947 passed ·  291 failed ·  0 timed out
```

The 291, by cause:

```
185  :probe::Outcome::* · :probe::Echo::*     positional — SKIPPED by the codemod (below)
 72  "the bare variant spelling is retired"   sites the rename did not reach
 22  "non-exhaustive: open-typed match needs at least one hash-destructure"   ⚠ A DIFFERENT CLASS
```

## ★★★ THE CAUSE — A NAME DECLARED FIVE WAYS, AND AN ACCEPTANCE ROW THAT COULD NOT SEE IT

`:probe::Outcome` is declared in **five test files with five different variant sets**:

```
probe_arc278_recv_outcome_wall.wat        :Message :Lost :Stopped
probe_arc209_c0b3bb_bounced_bounced.wat   :Bounced :Served
probe_arc278_service_max_frame_bytes.wat  :Message :Lost :Stopped :Closed
probe_arc170_m1_teeth_revoked.wat         :Bounced :Served
probe_arc278_dead_child_speaks.wat        :Message :Lost :Closed
```

Every construction of them is SKIPPED, and the sites are live code — `(:probe::Outcome::Message)`
needs `{}`, `(:probe::Outcome::Lost expr)` needs `{:field expr}`.

⛔ **AND SKIPPING IS IDEMPOTENT.** EXPECTATIONS row 8 asked for "re-run; the second run changes
nothing" — a codemod that does nothing satisfies that perfectly, and it was reported PASSED
honestly. **The row was the defect, not the report.** M2's residue 3 saw one instance of this
(`UNRESOLVED :probe::Outcome::Served`) and it was not generalized.
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

## TWO CANDIDATE MECHANISMS — DETERMINE WHICH, DO NOT ASSUME

The codemod already HAS the right shape: `stdlib-fmap` (:286, global) plus `fmap-for-src` (:293,
per-file). So something defeats the per-file path. Either:

**(A) redefinition across files.** The codemod evaluates declarations in one long-lived runtime; the
second file redeclaring `:probe::Outcome` hits a `DuplicateDefine` refusal, so files 2-5 get an
EMPTY fmap. (Consistent with M2's "count mismatch or missing decl".)

**(B) merge order.** The global map is consulted first and shadows the per-file entry.

⛔ **A skipped site must SCREAM, not pass silently.** Whatever the mechanism, the codemod must print
`[ctor] UNRESOLVED <head> <file>:<line>` for every construction it declines to rewrite — the arm
codemod's own precedent (`[match-arm] UNRESOLVED`). A tool that silently declines is a tool whose
idempotence proves nothing.

## THE WORK

1. Make every declined construction **report itself**. Run once; the report is the true worklist.
2. Determine (A) or (B) by measurement, fix it, re-run.
3. The 72 remaining bare-spelling sites: the rename pass over the same worklist.
4. **Diagnose the 22 `non-exhaustive: open-typed match needs at least one hash-destructure`** — this
   is NOT the migration's error class. Say whether it predates the campaign or was introduced.
5. Report the floor delta per population.

## ⛔ AND ONE THING FROM RELAND 1 MUST COME OUT

`src/services/verbs.rs read_via_stdin` accepts a `HashMap` where `ReadFrameOutcome::Frame`'s `text`
field is declared `String`, and digs `:text` out of it. **Remove that arm.** The wat producer is
correct (`stdio.wat:312` emits `{:text line}`), so a HashMap arriving there means a producer is
still building map-as-payload — the exact defect two stones exist to delete. The accommodation makes
it permanently invisible.

Let it fail loudly; the error names the producer in one run. That is the cure — the constructor's
refusal, not the consumer's tolerance. `[[feedback_the_wall_was_the_fix_not_the_fold]]`

## STOP TRIGGERS

- **STOP-1 — a site is declined silently.** Every decline prints UNRESOLVED with head, file, line.
- **STOP-2 — idempotence is offered as evidence of completion.** It is necessary, not sufficient.
  The bar is the fixture-local error count and the floor.
- **STOP-3 — the `read_via_stdin` HashMap arm is kept.** Remove it; report what fails.
- **STOP-4 — the 22 non-exhaustive failures are folded into "migration noise."** Different reason,
  different mechanism; diagnose and name it.
- **STOP-5 — a `.wat` site is hand-edited.** R21.
