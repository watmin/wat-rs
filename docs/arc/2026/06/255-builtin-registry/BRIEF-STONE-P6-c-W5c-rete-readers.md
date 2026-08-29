# STONE P6-c-W5c — the four rete readers whose purity nobody has ruled

> Wave 5c closes the rete verbs that are neither predicates (W5a) nor session mutators (W5b).
> Read both first. The firing family is NOT here — see
> `NOTE-the-firing-family-is-dual-implemented.md`.

## The four

```
:wat::rete::lower            crate::rete::expr_ir::eval_lower
:wat::rete::collect-rules
:wat::rete::step-payload
:wat::rete::axis-violation
```

**Pre-checked — verify, don't redo:** all four SHAPE=fits; all four on `KNOWN_UNREVIEWED`; none on
`FROZEN_CHECKER_DEBT_LEDGER`; **none has a wat `defn` twin and none shares its arm** — I checked
that with an exact pattern after a loose `\b` one falsely flagged `axis-violation` by matching
`axis-violation-message`. Confirm it yourself.

## ★ WHAT IS DIFFERENT: purity is FREE to be honest here, and that is a trap

W5a could not declare any verb Effectful without a red; W5b had to widen the prefix to declare six.
**`:wat::rete::` is now on `effectful_by_prefix`, so either answer passes the gate.** No gate will
tell you that you got it wrong.

**These four were deferred from W5a precisely because I had NOT verified they are read-only.** Their
names suggest reading; `lower` lowers an expression to IR and `collect-rules` collects — both could
plausibly build, cache, or intern, which is what made `arm-session`'s family Effectful. **Ground each
on what its body does. Do not inherit W5a's Pure or W5b's Effectful because of the namespace** — W5b
already proved the namespace is not the unit: two of its six were Effectful for caller-supplied-code
reasons, not session-mutation reasons, and got `Nondeterministic` where the other four got
`Deterministic`.

## Predictions

```
KNOWN_UNREVIEWED       225 → 221    (−4, the GATE'S OWN line)
FROZEN_CHECKER_DEBT    53 → 53      (unchanged)
population             110 → 106    registry 418 → 422
effectful_by_prefix    UNTOUCHED    (already widened by W5b — no change either way)
```

## Row 2 carries over

W3 seven lies in ten · W4 one in three plus five stale sites · W5a none but two near-misses reported ·
W5b one lie **whose stale twin still sits in `check.rs`** (`"Pure: no Environment, no eval_inner on
fact-args"` on a verb now declared Effectful). Read every doc against its body; report matched,
corrected with before/after, or near-miss. **`check.rs` stays untouched** — its diff must be empty.

★ Purity decides the example: `Pure` AND `Deterministic` obliges a RUNNABLE `@example`.

## The commands (git-independent — the `git ls-files` form is broken for untracked files)

```bash
grep -rhcE '^[ \t]*#\[wat_intrinsic' --include=*.rs src crates | awk '{s+=$1} END {print s}'
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture 2>&1 | grep -aE "UNREVIEWED"
python3 wat-scripts/hunt/p6c-disposition-census.py --no-multisite 2>&1 | grep -aE "HOMEABLE|AWAITING|RULED OUT"
```

★ W5b's technique is worth reusing: take "before" numbers from a **real HEAD clone**
(`git clone --local` into scratch, build, run) rather than reverting the working tree. Every before
figure is then measured, not asserted — and no `git stash` is involved.

## STOP triggers — each REJECTS.

1. **A purity you would have to guess.** No gate will catch you here; that is why this is STOP-1.
2. **A verb turns out to have a wat `defn` twin or a shared arm.** It belongs with the firing family
   — report it and leave it.
3. **A doc contradicts its body and you cannot tell which is right.**
4. **You are about to edit `src/check.rs`.**

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK of all four: shape · return type · arity · dispatch sites · both ledgers ·
      AND whether a wat `defn` or shared arm exists (exact pattern, not `\b`).
 1. ★ FOUR RULINGS with disk citations. Instrument: AWAITING 75 → 71, population 110 → 106.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched / corrected (before+after) / near-miss.
 3. ★ REAL ARITY PUBLISHED — `metadata-of` for all four, before and after.
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after.
 6. ★ FOUR ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ `KNOWN_UNREVIEWED` 225 → 221 by the gate's own line. Debt ledger unchanged at 53.
 8. ★ PURITY GROUNDED — one sentence per verb on what its BODY does. Say explicitly, for each,
      whether it allocates/interns/caches anything that outlives the call.
 9. ★ `git diff --stat src/check.rs` EMPTY. Say it.
10. cargo build --release --all-targets — clean; warnings VERBATIM.
11. cargo nextest run --release -E 'test(rete) + test(intrinsic) + test(purity) + test(reflection)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table including the dual-impl check. The four rulings. Row 2's verdicts. Arity and
error quotes. The gate's `UNREVIEWED` line. Four purity groundings, each saying what outlives the
call. Then the honest deltas — especially any verb whose purity surprised you.
