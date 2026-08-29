# STONE P6-c-W6 — eight `:wat::core::` collection readers

> The first wave into `:wat::core::`, and it is deliberately the half that needs no widening.
> Read `NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md` first — it is why the
> higher-order verbs are not here.

## The eight

```
:wat::core::length     :wat::core::nth        :wat::core::find-last-index
:wat::core::empty?     :wat::core::reverse    :wat::core::range
:wat::core::last       :wat::core::rest
```

**Pre-checked — verify, don't redo:** all eight SHAPE=fits; **none has a wat `defn` twin** (exact
pattern, not `\b` — a loose one falsely flagged `axis-violation` last wave by matching
`axis-violation-message`); none is on `FROZEN_CHECKER_DEBT_LEDGER`; **five are on
`KNOWN_UNREVIEWED`** (`last`, `rest`, `reverse`, `find-last-index`, `range`) and three are not
(`length`, `empty?`, `nth`).

## ★ THE ONE QUESTION, AND A RED MEANS STOP

**Does this verb run code it did not write?** Three verbs have been ruled Effectful on exactly that
mechanism — `stream::next` (forces a thunk), `rete::eval-test`/`eval-insert` (evals caller
expressions), `rete::collect-rules` (shape-only filter, then invokes every match). **These eight
should not**: they read a collection and return a value.

⚠ **`:wat::core::` is NOT in `effectful_by_prefix`, and it must not be added.** The census asserts
`Effectful ⇒ effectful_by_prefix`, so an Effectful ruling here goes RED — and the honest widening is
unavailable, because `:wat::core::` is a mostly-pure namespace and widening it would make the guess
vacuous for the largest namespace in the language (see the NOTE).

**So a census red is STOP-1: report which verb runs caller code and stop.** Do not widen
`:wat::core::`. Do not declare a verb Pure to keep the gate quiet — W5c proved a wrong Pure ruling
here is invisible, and W2's rider named and rejected that exact temptation.

★ **Beware `nth`, `last`, `rest` on a STREAM.** If any of these accepts a lazy seq and forces it,
that is the `stream::next` mechanism and the answer is Effectful — which fires STOP-1. **Check the
body for stream/thunk handling specifically**; it is the most likely way this wave surprises us.

## ⚠ AND A PRE-CHECK I COULD NOT DO FOR YOU

I tried to pre-classify these by grepping their bodies for `eval_inner`/`apply_function` and **the
instrument was unreliable in both directions**: `eval_inner` appears in nearly every BINDING handler
because it evaluates its OWN arguments — ordinary call-by-value, not an effect (W5c's rider drew that
distinction explicitly) — while `map` and `filter` came back "clean" despite certainly applying a
caller's fn. **There is no grep for this. It requires reading the body**, which is row 8's work.

## Predictions

```
KNOWN_UNREVIEWED    221 → 216   (−5: last · rest · reverse · find-last-index · range)
FROZEN_CHECKER_DEBT 53 → 53     (unchanged)
population          106 → 98    registry 422 → 430
effectful_by_prefix UNTOUCHED — a change here is STOP-1, not a chore
```

★ **Arity ≥ 5 note (W5c's structural finding):** a verb declaring 5+ wat args plus the
`env`/`sym`/`list_span` tail exceeds clippy's 7-argument limit. None of these eight should reach it —
if one does, use `#[expect(clippy::too_many_arguments)]` with a reason, following
`src/intrinsic/kernel/resource.rs:411` and `src/rete/step_payload.rs`. Never `#[allow]`.

## Row 2 carries over

W3 seven lies in ten · W4 one plus five stale sites · W5a two near-misses · W5b one lie whose stale
twin still sits in `check.rs` · W5c clean. Read every doc against its body. **`check.rs` stays
untouched** — its diff must be empty.

★ Purity decides the example: `Pure` AND `Deterministic` obliges a RUNNABLE `@example`, and a doc
`@arg`/`@ret` type must match `check.rs`'s registration **verbatim**, even where the registration is
less precise than the truth (W5c corrected a doc *down* to match disk).

## The commands (git-independent — `git ls-files` misses untracked files)

```bash
grep -rhcE '^[ \t]*#\[wat_intrinsic' --include=*.rs src crates | awk '{s+=$1} END {print s}'
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture 2>&1 | grep -aE "UNREVIEWED"
python3 wat-scripts/hunt/p6c-disposition-census.py --no-multisite 2>&1 | grep -aE "HOMEABLE|AWAITING|RULED OUT"
```

★ Take "before" numbers from a real HEAD clone (`git clone --local` into scratch, build, run) — W5b's
technique, reused by W5c. Every before figure measured, no `git stash`.

## STOP triggers — each REJECTS.

1. **A verb runs caller code / forces a thunk.** Report it; do not widen `:wat::core::`, do not
   declare it Pure.
2. **A doc contradicts its body and you cannot tell which is right.**
3. **A wat `defn` twin or shared arm exists.** It belongs with the dual-impl family.
4. **You are about to edit `src/check.rs`.**

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: shape · return type · arity · dispatch sites · both ledgers · dual-impl
      (exact pattern). Disagreements reported BEFORE any edit.
 1. ★ EIGHT RULINGS with disk citations. Instrument: AWAITING 71 → 63, population 106 → 98.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched / corrected (before+after) / near-miss.
 3. ★ REAL ARITY PUBLISHED — `metadata-of` for all eight, before and after.
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after — include one call per verb on a NON-trivial
      collection, not only the empty case.
 6. ★ EIGHT ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ `KNOWN_UNREVIEWED` 221 → 216 by the gate's own line. Debt ledger unchanged at 53.
 8. ★ PURITY GROUNDED — one sentence per verb answering: does it run code it did not write, and
      does it force a lazy seq? Both answers explicit, for all eight.
 9. ★ `git diff` on `effectful_by_prefix` EMPTY, and `git diff --stat src/check.rs` EMPTY. Say both.
10. cargo build --release --all-targets — clean; warnings VERBATIM.
11. cargo nextest run --release -E 'test(core) + test(intrinsic) + test(purity) + test(seq)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The eight rulings. Row 2's verdicts. Arity and error quotes. The gate's
`UNREVIEWED` line. Eight purity groundings, each answering both questions. Then the honest deltas —
especially any verb that touches a stream.
