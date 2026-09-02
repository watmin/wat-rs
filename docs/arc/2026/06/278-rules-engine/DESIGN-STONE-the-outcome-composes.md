# DESIGN — the outcome composes

Drawn 2026-09-01. **Not struck.** The ordering at the bottom is the ruling that governs the
next four pieces of work; this document is also the design for the first of them.

## Why

Two sites in the queue insert a **one-millisecond timer into the message path**, and neither
wants a delay:

```wat
;; wat-scripts/queue/sqs.wat:238-241 — send, with a consumer parked
(if (empty? waiters)
  (Outcome::Reply s' Ok)
  (Outcome::ReplyAndArm s' Ok [(Alarm :after (Millisecond 1) :op :-tick)]))
```

It wants to *reply `Ok` to the sender **and** hand the message to the waiter*. Replying to the
caller is `Reply`; waking a waiter is `ReplyTo`. **No outcome does both.** So it replies, stashes
the rest of its intent in state, and sets a clock to come back and finish.

The second site says so in its own margin (`sqs.wat:350-351`):

> *"flush outbox on a 0-ns tick when we both ReplyTo and need to re-arm (**no ReplyToAndArm**)."*

Both were originally `Nanosecond 0` — the spelling of *"come back immediately"* — and both were
clamped to `Millisecond 1` when the sane circuit found that a duration-0 `after` never fires at
process tier.

**The root is not the timer.** `Outcome` (`wat/service.wat:72-81`) hard-codes combinations of four
independent things as six named variants:

```
Reply · Stop · NoReply · ReplyAndArm · NoReplyAndArm · ReplyTo
```

Four axes, six of the combinations. The two the queue needs — `Reply`+`ReplyTo` and
`ReplyTo`+`Arm` — are not among them, so **a clock is standing in for the word *and***.

A seventh variant is the wrong fix. That patches the stem; next month someone needs
`Stop`+`ReplyTo` and it is an eighth.

## What it delivers

An `Outcome` with no missing combinations, and a second one that makes a whole class of arm
unable to lie about having a caller.

## The shape

Uniform vectors would *create* illegal states, so the cardinalities are not flattened:

| axis | cardinality | why |
|---|---|---|
| state | exactly 1 | an arm always yields its next state |
| reply to **my** caller | **0 or 1** | one invocation, at most one reply. A vector here makes "reply twice to the same conn" representable — a protocol violation with no meaning |
| sends to **other** conns | 0..N | genuinely many; each names a different parked conn |
| arms | 0..N | genuinely many |
| lifecycle | exactly 1 of {continue, stop} | not a field — it governs what the other fields may contain |

So the enum survives on the **lifecycle axis only**; the rest become fields:

```wat
(:wat::core::defenum :wat::service::Outcome :- [S R O] :wat::enum::Pure
  :Continue [state <- :S
             reply <- (:wat::core::Option :- [:R])
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])
             arms  <- (:wat::core::Vector :- [(:wat::service::Alarm :- [:O])])]
  :Stop     [state <- :S
             reply <- (:wat::core::Option :- [:R])
             sends <- (:wat::core::Vector :- [(:wat::service::Directed :- [:R])])])
```

- **`Stop` has no `arms`.** Scheduling future work on a terminating service is incoherent; it gets
  no form rather than being accepted and ignored.
- **`Stop` gains `sends`.** Today `Stop [state, reply]` cannot answer parked waiters at all.

★ **Hypothesis, not a claim.** The unverified substrate finding — *"4 parked waiters hang
`Admin::Stop`"* (`SCORE-the-sane-circuit.md`) — may be exactly this hole: a stopping queue cannot
reply to its parked receivers because the stopping outcome has nowhere to put the sends. Test it;
do not assume it.

### Two outcome types, not one

The **input** side already splits, and the substrate states why (`wat/service.wat:127`): an
internal arm receives a `SelfInvocation`, *"never an `Invocation` (it has no connection, so it has
no `conn-id` field)"*. The **output** side does not split — so an internal arm, which has no
caller, can construct `Outcome::Reply` for one.

```
public   arm  [s ctx req]  ->  Outcome      (reply <- Option<R>)
internal arm  [s ctx]      ->  SelfOutcome  (no reply field at all)
```

`SelfOutcome::Continue {state, sends, arms}` / `SelfOutcome::Stop {state, sends}`. Then "an
internal arm replies to its caller" has no representation.

⚠ **This rests on an argument from absent grep.** A search of `src/check.rs` for any rule keyed on
internal-arm outcomes found none, which is why the hole is believed live — but absence of a grep
match is not proof of absence of a rule. **Probe it first** (ten lines: an internal arm returning
`Reply`; does it check, and does it run?). If a rule already exists, `SelfOutcome` is closing a
theoretical hole, not a live one, and the stone should say so.

## The one contract decision

**`reply` is `Option<R>`, never a vector.** At most one reply per invocation is the protocol, and
the type is where that is enforced.

## Files touched

`wat/service.wat` (the enum and every serve-loop arm that matches it), and every `:impls` arm in
the corpus that constructs an `Outcome`. The `.wat` corpus migration is a **wat-fix codemod**
(`wat/fix.wat`, recorded under `wat-scripts/fixes/`) — never hand-edits, never python or sed. Note
the **BOOTSTRAP/STASH-DANCE** header in `fix.wat`: `wat/service.wat` is a frozen stdlib member, so
the tool cannot boot to fix itself once the file is mid-change.

## Out of scope = REJECTED

- **Making `after 0` illegal.** Correct, and it comes *after* this — see the ordering.
- **Deleting the naps in the circuit.** Downstream of this; a parked reply replaces them.
- **The store swap.** Independent lane; see the ordering.
- **A general effects/queue abstraction for services.** One consumer, no second user.

---

# THE ORDER — ruled 2026-09-01

Four pieces of work. **1–3 are one dependent chain; 4 is independent of all of them** and is the
only measured 6× on the table.

### 1. `Outcome` composes — *this document*
The stone. Deletes both `Millisecond 1` hacks by making *"and also"* expressible, and is a
precondition for 2 and 3.

### 2. Probe the internal-arm `Reply` hole
Ten lines. Settles whether `SelfOutcome` closes a live hole or a theoretical one. Cheap enough to
run before 1 lands, and it changes what 1 must claim.

### 3. `after 0` becomes illegal
The kernel is **correct** — `timerfd_settime` with `it_value = 0` disarms; that is POSIX. The
defect is that `after` presents as locus-agnostic and is not: thread fires, process is silent, no
diagnostic. The derivation is clean — **an Alarm schedules *future* work; zero is not future** — so
once nothing legitimately needs "now", zero should have no form.

**Order is load-bearing.** Forbidding zero *today* breaks the queue, because zero is currently how
it spells *and also*. Compose first, then forbid.

Where the constraint lives: a validating `Alarm` constructor is a check at construction time; a
`PositiveDuration` with no zero constructor is no-form. The second costs more — `Duration`
constructor sites number **7 in `wat/`, 32 in `wat-scripts/`, 14 in `tests/`, 26 in `src/`** — and
`:wat::kernel::after` takes the same type with the same zero problem. A good share of those 32 are
naps that item 1 deletes anyway. **Unchecked:** whether a *negative* duration is representable
there.

### 4. The store measurement — independent, and the only recorded 6×
`SCORE-perf-2-store-read-path.md:51`, written 2026-09-01:

> *"sqlite runs the same circuit in 43 s — six times faster than mem, because it has a real
> database underneath doing indexed writes."*

Mem was 257 s at that moment. `DESIGN-STONE-the-indexed-vector-update.md:77` then placed sqlite
under *"Out of scope = REJECTED"*, and perf-3 optimised the persistent vector instead.

`sqlite-store` satisfies the **same** surface (`wat/query/sqlite-store.wat:283-284`), and the queue
holds its store as a peer of that surface, not of a concrete service
(`wat-scripts/queue/sqs.wat:104`, `:115`) — so the backend is genuinely a parameter to the queue.
The concreteness leaks only in the *caller's* handle vector: **19 `mem-store` mentions in
`circuit.wat`**, because a `Handle` is per-service.

⚠ **`wat-scripts/scratch-pad/probe-circuit-sqlite.wat` is a two-stone fossil** — it predates both
the sane circuit and async publish, and has **no drain wait at all**. Run today it returns
`total=879` of 8000. Any number taken from it as-is is a measurement of one thing quoted as a claim
about another. It needs porting before it can be believed.

---

## Why this order, and not the one I argued for first

I spent four messages proposing fan-out concurrency and timer tuning **before** measuring the
store, with a 6× result already recorded in my own SCORE from the day before, in a section titled
*"Recorded, not chased."* The ordering above puts the measured term in its own lane so it cannot be
starved by the designed ones again.
