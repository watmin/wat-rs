# STONE P6-c-W3 — the reflection surface comes home (ten verbs)

> Wave 3, and the largest yet — double W2. Read `BRIEF-STONE-P6-c-W2-stream-program-stdlib.md`
> first; its deltas are requirements here.

## The ten

```
:wat::runtime::lookup-define          eval_lookup_define            runtime.rs:13184
:wat::runtime::signature-of-defn      eval_signature_of_defn        runtime.rs:13291
:wat::runtime::signature-of-fn        eval_signature_of_fn          runtime.rs:13423
:wat::runtime::return-type-of         eval_return_type_of           runtime.rs:13477
:wat::runtime::body-of                eval_body_of                  runtime.rs:13562
:wat::runtime::rename-callable-name   eval_rename_callable_name     runtime.rs:13836
:wat::runtime::extract-arg-names      eval_extract_arg_names        runtime.rs:14021
:wat::runtime::extract-arg-types      eval_extract_arg_types        runtime.rs:14134
:wat::runtime::argv                   eval_runtime_argv             runtime.rs:20023
:wat::runtime::current-thread         eval_runtime_current_thread   runtime.rs:20051
```

**Pre-checked by the orchestrator — verify, don't redo:** all ten are SHAPE=fits; **all ten return
`Result<Value, EvalBreak>`** (W2's blocker does not apply — I checked every one, because the census
reads parameter types and never the return type); **none is in `KNOWN_UNREVIEWED`**; all ten are
checker-known; each carries an inline hand-rolled arity guard.

⛔ **`metadata-of`, `field-names-of`, `field-types-of` are NOT in this wave.** They are the three
`:wat::runtime::` verbs the checker does NOT know, so homing them ADDS to
`FROZEN_CHECKER_DEBT_LEDGER` — a real act that deserves its own stone. Affirmative cut, not a
deferral. ★ And `metadata-of` staying an arm is convenient: it is the instrument this wave's
acceptance uses.

## ★ WHAT IS DIFFERENT ABOUT THIS WAVE — the docs become USER-FACING

These are the **reflection surface** — the verbs arc 255 has spent itself making honest. Stone P6-a
established the rule: **a fn named by `#[wat_intrinsic]` has USER-FACING documentation**;
`show-source` and `render-doc` print it. The moment P6-a made `if`'s source reachable it published
**two inverted doc comments buried since arc 258.4**.

So: **read each handler's existing prose against its body before you carry it into a `///` block.**
Expect to find at least one lie. Finding one is a SUCCESS of this wave, not a delay — fix it, and
**report it prominently with the before/after text**. A stale comment you migrate unread becomes a
shipped lie with your name on the commit.

## Purity is per-verb here, and two of them are not like the other eight

Eight are reads of the symbol table / AST. **`argv` reads process arguments** and **`current-thread`
reads the running thread** — ground each yourself. W1 and W2's coupling holds: **decide purity
first, because `Pure` AND `Deterministic` obliges at least one RUNNABLE `@example`;
`@example-norun` is refused by `purity_mandated_examples`.**

## ⛔ The commands, because absolutes have been wrong twice and that is MY fault

Both prior riders reported a "drifted baseline" that did not exist — each invented its own grep. Use
exactly these:

```bash
# registry attribute count (anchored — an unanchored grep counts `//!` prose and returns ~456)
grep -rhcE '^[ \t]*#\[wat_intrinsic' $(git ls-files '*.rs') | awk '{s+=$1} END {print s}'
# the SAME count at HEAD, for the delta
git stash list >/dev/null; for f in $(git ls-tree -r HEAD --name-only | grep -E '\.rs$'); do git show HEAD:$f; done 2>/dev/null | grep -cE '^[ \t]*#\[wat_intrinsic'
# the unreviewed ledger — the GATE'S OWN len(), never a grep of the array
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture 2>&1 | grep -aE "UNREVIEWED"
# the population
python3 wat-scripts/hunt/p6c-disposition-census.py --no-multisite 2>&1 | grep -aE "HOMEABLE|AWAITING|RULED OUT"
```

**Report the DELTA and the command you ran.** If an absolute disagrees with mine, the delta is what
we trust — and say so rather than declaring the baseline drifted.

## STOP triggers — each REJECTS.

1. **A doc comment contradicts its body and you cannot tell which is right.** Report both, home
   nothing for that verb.
2. **A purity you would have to guess** — especially `argv`/`current-thread`. Leave it unhomed.
3. **A verb turns out multi-site or not checker-known.** My pre-check was wrong; that is a finding.
4. **Ten is too many and quality would slip.** Home fewer, say which and why. **Eight done properly
   beats ten rushed** — W2 shipped four instead of five and was right to.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK of all ten: shape · return type · checker-known · KNOWN_UNREVIEWED ·
      dispatch sites · declared arity. Disagreements reported BEFORE any edit.
 1. ★ TEN RULINGS with disk citations. Instrument: AWAITING 103 → 93. Paste the summary.
 2. ★ EVERY DOC COMMENT READ AGAINST ITS BODY. State for each: matched, or corrected — with the
      before/after text for every correction. This is the row most likely to earn its keep.
 3. ★ REAL ARITY PUBLISHED. `metadata-of` for all ten, before and after. State each expected arity
      from the guard you deleted.
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after. `git show HEAD:<path>` — never `git stash`.
 6. ★ TEN INLINE ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ BOTH FROZEN LEDGERS UNCHANGED — `KNOWN_UNREVIEWED` by the gate's own line,
      `FROZEN_CHECKER_DEBT_LEDGER` at 50. A move is a finding.
 8. ★ PURITY GROUNDED — one sentence per verb; `argv` and `current-thread` in full.
 9. ★ Population 138 → 128. Registry delta +10, with the command you used.
10. cargo build --release --all-targets — clean; report any warning VERBATIM.
11. cargo nextest run --release -E 'test(runtime) + test(intrinsic) + test(purity) + test(reflection)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The ten rulings. **Row 2 in full — every doc comment's verdict, and the
before/after text of every correction.** `metadata-of` and arity quotes before and after. The gate's
`UNREVIEWED` line before and after. Ten purity justifications. Then the honest deltas — especially
any doc lie you found, and whether ten was too many.
