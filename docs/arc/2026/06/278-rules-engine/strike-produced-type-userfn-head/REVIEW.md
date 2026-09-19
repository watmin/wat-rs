# REVIEW — the DIVERGENCE holds and is mutation-proved. The REFEREE has no instrument.

## What I re-ran myself, and what held

| claim | my independent result |
|---|---|
| oracle names the fn | **HOLD** — `["pt::first-rate"]` vs anchor `["pt::Rate"]`, driven via `cargo run --release --bin wat` |
| arm 1 native MEASURE | **HOLD** — `pt::Rate`, oracle half driven in-process, no hardcoded print |
| ⭐ arm 2 facts differ | **HOLD** — I ran the scratch: native `[0 1 1]`, oracle `[0 1 0]`. **The oracle drops `Out`.** |
| the mechanism | **HOLD** — `ORACLE STRATA: {"a2::mk-rate" 1}`. The stratum lands on the FUNCTION, so `Rate` is never raised and the consumer sits at 0 and is never re-fired. |
| the purity fence is not the blocker | **HOLD** — `stratify.rs:418-424` says exactly that: *"constructs a record at all -> `kwargs-construct` is not pure`. **FALSE today**… Driven 2026-08-28."* |
| floor | **HOLD** — `5475 tests run: 5475 passed (2 slow), 22 skipped` |

**Mutation-proved by me, live source, restored:** deleting the `SymbolTable` resolution in
`produced_type` (return the bare head) REDs **both** tests —

```
left:  (["pt::first-rate"], ["pt::first-rate"])
right: (["pt::Rate"],       ["pt::first-rate"])
```

— and the facts test REDs too, because native then drops `Out` as well. That second red is the
better half of the proof: **native's correct answer comes precisely from the resolution the oracle
lacks.** Mechanism pinned, not inferred.

⭐ **This is a genuine engine-level defect in derived FACTS**, the first of this session. Rowed, not
disputed.

## ⛔ THE REFUTATION — row 6 is the load-bearing claim and it has no instrument

Row 6 reads: *"Clara referee: **HOLD.** Same shape, user-fn inlined as a Rate constructor:
`[0 1 1]`. Matches native."*

**No Clara model exists on disk.** `find /home/john/work/holon -name '*.clj' -newermt '2026-09-07 09:00'`
returns nothing; `clara/` is the untouched August upstream checkout; the scratch `.wat` contains no
Clara reference. So the number cannot be re-run by anyone, including you tomorrow.

That matters more here than anywhere else in this strike, because **row 6 is the only row that says
which engine is wrong.** Rows 1-5 establish that native and oracle disagree; on their own they are
symmetric. `[[when-two-engines-disagree-neither-is-the-referee]]` — the whole builder hierarchy is
Clara → oracle → native, and this is the moment it is load-bearing.

And the tree has already been bitten by exactly this shape: the DESIGN's own warrant for the grid
axis was that `probe_arc278_oracle_accumulate_supersedes`'s Clara figure is *"a dated comment,
hand-derived once"*. An uncommitted Clara run is that same artifact before it even reaches a
comment. `[[a-metric-without-its-instrument-cannot-be-rechecked]]`.

⚠ I am **not** disputing the number. Reasoning says native is right, and `16f504e14` is precedent
for the oracle being the wrong one. But "reasoning says" is precisely what a referee exists to
replace.

## The work

**Commit the Clara model that produces `[0 1 1]`.** A static `.clj` beside the scratch (or under
`wat-scripts/perf/grid/` if it fits that shape), plus the exact command in its header, so the
referee is re-runnable. Then re-run it and quote the output verbatim in the SCORE.

Two modelling points the file must state in its own header, because they are judgement calls a
future reader has to be able to check:

1. **The user fn is INLINED.** Clara has no `:then`-head-is-a-fn concept, so `mk-rate` becomes a
   direct `(->Rate k)` in the RHS. State that this is faithful because `mk-rate`'s body is exactly
   that construction — and if it is not exactly that, say what differs.
2. **`Bad` never fires** (`k=2` against `Src(1)`), so the negation is vacuously true. Say so, and
   confirm Clara's `Rate` is derived for the same reason wat's is.

## STOP triggers

1. **If Clara does NOT return `[0 1 1]`** — STOP and surface it immediately. If Clara matches the
   ORACLE, the direction inverts and native is the defect; if it matches neither, the model is
   wrong. Either outcome outranks everything else here.
2. **If Clara cannot express the shape** — STOP and say so with the error. Do not inline something
   that changes the rule's meaning to get a number.
3. Still no cure: do not touch `produced_type`, `rule-produces`, or `purity.rs`. Direction is not
   settled until the referee is re-runnable.

## Not in scope, but rowed here so it is not lost

`tests/rete/probe_arc278_then_user_forms_userfn.wat:12-22` states that a constructing user fn is
refused as impure. `stratify.rs:418-424` records that this is **FALSE today**, driven 2026-08-28.
**Two places in the tree disagree, and I drew this strike's outcome-2 hypothesis off the stale
one.** The probe header needs correcting — its own strike, after this one lands.
