# DESIGN — the harness takes a record

Builder: *"i noticed we have a bunch of really long argv now…… it may be best… to make these a record
and we can use record accessors instead of positions… we can also echo/printf into the program the
record shapes… `#u/Input {:arg1 val1 :arg2 val2 …}` … seeing something with 10+ fields screams we need
a structured holder."*

**Drawn 2026-09-14. NOT STRUCK.** ⛔ **Sequenced AFTER `the-topic-can-be-slow`**, which is in flight
and appends argv slots 17/18. Not a conflict — this stone sweeps every slot including those two, and
refuting a live strike over a representation change would be the second time today that cost a working
strike.

## WHY — and it is not ergonomics

The positional scheme has a **measured defect**, which is the argument this stone rests on:

⛔ **Four sites overload `0` to mean "the default", so three knobs cannot be set to zero at all.**

```
circuit.wat:2920   vis-ms > 0 is the sweep knob … 0 means [the default]
circuit.wat:2929   ms → ns; 0 = the default
circuit.wat:3666   0 means the default, NOT "no redelivery"
circuit.wat:3699   inbox-cap … Optional; 0 means 64
```

`:3666` says it outright. **`vis-ms`, `inbox-vis-ms` and `inbox-cap` have an unreachable value**, and
`vis-is-swept-not-chosen` is a stone whose whole subject was sweeping `vis`. A sweep that cannot
express zero has a hole in it that no one can see from the call site.

The scale, listed rather than counted:

```
:fanout::run-with parameters   18   (n m j p rate seed drop-check-bp drop-mark-bp drop-seed
                                     drop-after? drop-recv-bp drop-ack-bp sub-cap fill-first?
                                     vis-ms inbox-vis-ms inbox-cap chaos-bp)
CLI argv slots in use          15   (argv 2–16) — +2 in flight = 17
live callers                    4 test files + scripts/capped.sh
```

⚠ My first count of `run-with`'s parameters was **17**: a fixed window clipped `chaos-bp`. The same
fixed-window defect once made an enum census miss the very enum that motivated it. The 18 above is
from a listing, not a count.

## ⛔ THE ONE CONTRACT DECISION

**Required fields for what a run cannot mean without; `Option` fields for every knob that has a
default — and `None` means "default" while `(Some 0)` means zero.**

That is the whole correctness win. A record with all-required fields would be tidier to read and would
**preserve** the defect, because every invocation would have to name a value and the "unset" case would
go back to being spelled `0`. The distinction between *absent* and *zero* is what the positional scheme
cannot express, and it is the reason to do this at all.

```
(:wat::core::defrecord :user::Input
  [n m j            <- :wat::core::i64                          ;; required — a run is meaningless without
   sub-cap          <- :wat::core::i64
   fill-first?      <- :wat::core::bool
   vis-ms           <- (:wat::core::Option :- [:wat::core::i64]) ;; None = default, Some 0 = ZERO
   inbox-vis-ms     <- (:wat::core::Option :- [:wat::core::i64])
   inbox-cap        <- (:wat::core::Option :- [:wat::core::i64])
   chaos-bp         <- (:wat::core::Option :- [:wat::core::i64])
   …])
```

## The mechanism already exists, and is the tree's standard shape

`(:wat::kernel::readln)` → `ReadlnOutcome::{Datum|Eof|Stopped}`. **Every recorded migration** under
`wat-scripts/fixes/` uses it, and `angle-brackets-to-binder.wat:282` names it
*"file/stdin harness — identical shape to every recorded migration"*. Those read an EDN **vector**;
this stone reads an EDN **record**. Same primitive, richer datum.

```
printf '#user/Input {:n 2000 :m 4 :j 3 :sub-cap 8192 :fill-first? true :vis-ms 1000}\n' \
  | ./target/release/wat wat-scripts/fanout/circuit.wat
```

⭑ Note the namespace: the builder wrote `#u/Input`; wat renders `:user::Input` as `#user/Input`, which
matches `:user::main` already in the file. Naming is the builder's to overrule.

## Out of scope = REJECTED

- **Converting `sqs.wat` / `sns-fanout.wat` / the codemods.** The codemods already take a stdin datum
  and are fine. One harness, proven, before a family.
- **Keeping a positional fallback.** Two input paths is two code paths and two sets of defaults, and
  the second one rots. ⚠ The cost is real and stated: **every existing invocation string in the SCOREs
  stops working.** Those are historical records and are NOT rewritten — a SCORE records what was run
  at the time, and editing them to match a later interface is exactly the map-drift `curare` forbids.
- **Making `p` reachable.** Still called from nowhere (`FINDING-we-cannot-induce-a-slow-peer` §3). It
  becomes a *field* here, which removes the excuse, but wiring and measuring it is its own stone.

## Blast radius

```
wat-scripts/fanout/circuit.wat   the defrecord, main's readln, run-with's signature or a record pass-through
tests/services/probe_ex001_fanout.rs · probe_chaos_gate_has_teeth.rs
  · probe_arc278_sane_circuit.rs · probe_async_publish.rs
scripts/capped.sh                verify — it may be arg-agnostic
docs/**                          0 — historical invocation strings stay as written
```

## Trap-doors named up front

1. ⛔ **`0` currently means "default" in four places. Converting them to `None` is a BEHAVIOUR change
   at every call site that passed `0` meaning "I don't care".** Each of the four needs its intent read
   individually — a blanket `0 → None` is the fallback-that-collapses-failures defect in a new costume.
2. ⚠ **The floor's tests are the callers.** A wrong conversion turns a green test into a green test
   measuring something else. Diff the *reported counters*, not just pass/fail.
3. **`readln` blocks.** A test that forgets to pipe input hangs rather than failing — and the floor's
   terminate wall is 30 s. Prefer a fixture that always provides stdin.
4. **`ReadlnOutcome::Eof` / `::Stopped` are real arms.** The migrations raise on both; that is correct
   for a codemod. For the harness, decide deliberately and say which.
5. ⭑ **Do not rewrite the SCOREs.** Their invocation strings are evidence of what was run, not
   documentation of a current interface.
