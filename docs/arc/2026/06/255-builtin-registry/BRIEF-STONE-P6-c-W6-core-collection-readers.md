# STONE P6-c-W6 — SEVEN `:wat::core::` collection readers

> ⛔ **RE-CUT 2026-08-29. THIS BRIEF NAMED EIGHT AND WAS REFUSED — CORRECTLY.**
> The eighth was `:wat::core::find-last-index`, and it is **a HOF wearing a reader's name**:
> `(Vector :- [T], Fn(T)->bool) -> (Option :- [i64])`, calling `apply_function` once per element
> (`src/collection/transform.rs`, the `for (i, x) in xs.iter()` loop). W6's rider read the body,
> found it, and **stopped at STOP-1 with zero edits** rather than home seven while one was blocked.
> That refusal was right and it is why this brief now names seven. `find-last-index` moves to the
> **W7 HOF family**, parked behind the `effectful_by_prefix` question.
> ★ Nothing else about the wave changed. Read on.

> The first wave into `:wat::core::`, and it is deliberately the half that needs no widening.
> Read `NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md` first — it is why the
> higher-order verbs are not here.

## The seven

```
:wat::core::length     :wat::core::nth        :wat::core::reverse
:wat::core::empty?     :wat::core::rest       :wat::core::range
:wat::core::last
```

## Where each one lives (measured this session — verify, don't hunt)

```
length            src/runtime.rs:5386  -> eval_length            (same file, ~:16538)
empty?            src/runtime.rs:5390  -> eval_empty             (same file, ~:16619)
nth               src/runtime.rs:5742  -> eval_nth               (same file, ~:14967)
last              src/runtime.rs:5758  -> collection::transform::eval_vec_last
rest              src/runtime.rs:5764  -> collection::eval::eval_rest
reverse           src/runtime.rs:6011  -> collection::transform::eval_vec_reverse
range             src/runtime.rs:6014  -> collection::transform::eval_vec_range
```

**Pre-checked — verify, don't redo:** all seven SHAPE=fits; **none has a wat `defn` twin**
(measured: `grep -rn "defn :wat::core::<v>\b" --include=*.wat wat/` = 0 for all seven — use the
exact pattern, not `\b` alone; a loose one falsely flagged `axis-violation` last wave by matching
`axis-violation-message`); **none is on `FROZEN_CHECKER_DEBT_LEDGER`** (measured against
`src/intrinsic/mod.rs:689`, whose total is 53); **four are on `KNOWN_UNREVIEWED`**
(`last`, `rest`, `reverse`, `range`) and three are not (`length`, `empty?`, `nth`).

## ★ THE ONE QUESTION, AND A RED MEANS STOP

**Does this verb run code it did not write?** Three verbs have been ruled Effectful on exactly that
mechanism — `stream::next` (forces a thunk), `rete::eval-test`/`eval-insert` (evals caller
expressions), `rete::collect-rules` (shape-only filter, then invokes every match). **These seven
should not**: they read a collection and return a value.

⚠ **`:wat::core::` is NOT in `effectful_by_prefix`, and it must not be added.** The census asserts
`Effectful ⇒ effectful_by_prefix`, so an Effectful ruling here goes RED — and the honest widening is
unavailable, because `:wat::core::` is a mostly-pure namespace and widening it would make the guess
vacuous for the largest namespace in the language (see the NOTE).

**So a census red is STOP-1: report which verb runs caller code and stop.** Do not widen
`:wat::core::`. Do not declare a verb Pure to keep the gate quiet — W5c proved a wrong Pure ruling
here is invisible, and W2's rider named and rejected that exact temptation.

## ★ THE STREAM HAZARD — PRE-CHECKED THIS TIME, AND IT CAME BACK CLEAN

The eight-verb brief said *"beware `nth`, `last`, `rest` on a STREAM — if any forces a lazy seq that
is the `stream::next` mechanism"*, and admitted it had no instrument for it. **It has one now, and
the answer is structural rather than a reading of intent.**

**Stone 118.B4-iii — THE WALL — already closed this door for all of them.** Each of these verbs
routes through `StreamContainer::of_value` and a capability gate (`measurable()` for
`length`/`empty?`, `has_tail()` for `rest`, `nth_indexable()` for `nth`, `ordered()` for `reverse`).
Every gate is **false for Stream**, so the `StreamContainer::Stream` inner arm is a named
`unreachable!()` — dead by construction, `_`-free so a future capability change is a compile error —
and a Stream argument falls to a `TypeMismatch` whose message teaches `:wat::stream::next` instead.
`rest` **used to** force one cell to discard it; 118.B4-iii deleted that path explicitly, with the
comment *"the same cost as `next`, but the name hid the force; the wall closes that."*

`last` and `range` mention no Stream at all (23 and 32 lines; screened, zero hits).

★ **The screen was proven non-vacuous before it was believed:** the identical
`apply_function|wat__core__fn|Stream` screen returns **2** on `eval_vec_find_last_index` (the known
HOF it correctly catches) and **0** on `eval_vec_last`/`eval_vec_range`. It can see what it claims
to look for.

⛔ **THIS DOES NOT RETIRE STOP-1, AND IT IS NOT PERMISSION TO SKIP ROW 8.** It is an orchestrator's
pre-check on one mechanism, and a pre-check is a claim, not a proof. **Read every body yourself.**
If any of the seven runs caller code or forces a thunk by a route this screen could not see, that is
still STOP-1 and you still stop — the last rider's refusal is exactly why this wave exists in this
shape, and it will be right again if it fires.

## ⚠ AND A PRE-CHECK NOBODY CAN DO BY GREP

Grepping bodies for `eval_inner`/`apply_function` is **unreliable in both directions**: `eval_inner`
appears in nearly every BINDING handler because it evaluates its OWN arguments — ordinary
call-by-value, not an effect (W5c's rider drew that distinction explicitly) — while `map` and
`filter` came back "clean" despite certainly applying a caller's fn. The screen above is aimed at
one narrow mechanism and nothing more. **The general question requires reading the body**, which is
row 8's work.

## Predictions

```
KNOWN_UNREVIEWED    221 → 217   (−4: last · rest · reverse · range)
FROZEN_CHECKER_DEBT 53 → 53     (unchanged)
population          106 → 99    registry 422 → 429
effectful_by_prefix UNTOUCHED — a change here is STOP-1, not a chore
```

★ **Arity ≥ 5 note (W5c's structural finding):** a verb declaring 5+ wat args plus the
`env`/`sym`/`list_span` tail exceeds clippy's 7-argument limit. None of these seven should reach it —
if one does, use `#[expect(clippy::too_many_arguments)]` with a reason, following
`src/intrinsic/kernel/resource.rs:411` and `src/rete/step_payload.rs`. Never `#[allow]`.

## Row 2 carries over

W3 seven lies in ten · W4 one plus five stale sites · W5a two near-misses · W5b one lie whose stale
twin still sits in `check.rs` · W5c clean. Read every doc against its body. **`check.rs` stays
untouched** — its diff must be empty.

★ Purity decides the example: `Pure` AND `Deterministic` obliges a RUNNABLE `@example`, and a doc
`@arg`/`@ret` type must match `check.rs`'s registration **verbatim**, even where the registration is
less precise than the truth (W5c corrected a doc *down* to match disk).

★ A param-spec in any doc, example, or type you write is spelled **`(Head :- [T …])` and nothing
else** — arc 109 closed at `556b9c08f`; the bare `(Head T …)` and unmarked `(Head [T …])` forms are
now unrepresentable, and the wall will reject them.

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
   declare it Pure. ★ The pre-check above says none of the seven does — **if your read disagrees,
   your read wins and you stop.**
2. **A doc contradicts its body and you cannot tell which is right.**
3. **A wat `defn` twin or shared arm exists.** It belongs with the dual-impl family.
4. **You are about to edit `src/check.rs`.**

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: shape · return type · arity · dispatch sites · both ledgers · dual-impl
      (exact pattern). Disagreements reported BEFORE any edit.
 1. ★ SEVEN RULINGS with disk citations. Instrument: AWAITING 71 → 64, population 106 → 99.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched / corrected (before+after) / near-miss.
 3. ★ REAL ARITY PUBLISHED — `metadata-of` for all seven, before and after.
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after — include one call per verb on a NON-trivial
      collection, not only the empty case.
 6. ★ SEVEN ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ `KNOWN_UNREVIEWED` 221 → 217 by the gate's own line. Debt ledger unchanged at 53.
 8. ★ PURITY GROUNDED — one sentence per verb answering: does it run code it did not write, and
      does it force a lazy seq? Both answers explicit, for all seven. ★ For the six with a
      118.B4-iii gate, name the gate (`measurable`/`has_tail`/`nth_indexable`/`ordered`) you
      confirmed, so the answer cites the wall rather than repeating this brief.
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

Your pre-check table. The seven rulings. Row 2's verdicts. Arity and error quotes. The gate's
`UNREVIEWED` line. Seven purity groundings, each answering both questions and naming its gate. Then
the honest deltas — including anything the orchestrator's stream pre-check got wrong.
