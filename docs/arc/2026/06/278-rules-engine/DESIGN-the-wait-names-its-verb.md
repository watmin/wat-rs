# DESIGN — the wait names its verb

**Stone B.** The queue stops spelling a mode as a magnitude.

> Builder, 2026-09-03: *"remove this sleep 0 bullshit from my language….. we annihilate all
> deadlocks on contact - they never survive us"*

## WHY

`sqs.wat:58` declares `wait-ns <- :wat::core::i64`. `sqs.wat:487` forks on it:

```wat
(:wat::core::if (:wat::i64::<= wait 0)   ;; ← reply empty immediately, never park
  ...                                     ;; ← else build a Waiter with a deadline
```

**One i64 selects between two operations.** `0` is not a short wait; it is a different verb — a
non-blocking sweep. The name announces only the other one, and the file already owns the right word
and does not use it (`:user::park-receive!` at `:859`, *"a parked receive"* at `:942`).

Cost on the record: **two deadlocks.** `dfacde23c` fixed the services and left the helpers on
`wait-ns 0`; the second is live and reproducible at
`wat-scripts/scratch-pad/probe-refused-retry-self-consumes.wat` (`gap=300 → SPINS-FOREVER`, 3/3).
Every spin loop in the corpus exists because a receive won't block.

Stone A removed zero from the *language*. **This removes it from the queue's protocol.**

## WHAT IT DELIVERS

Inside `(defsurface :queue::Queue :nature :wat::kernel::Peer)`:

```wat
(:wat::core::defenum :queue::Queue::Wait :wat::enum::Pure
  :Immediate []
  :UpTo [d <- :wat::time::NonZeroDuration])
```

and `ReceiveRequest`'s `wait-ns <- :wat::core::i64` becomes **`wait <- :queue::Queue::Wait`**.

The `-ns` suffix goes with it: it existed only because an i64 could not carry its unit. The call
sites then read as what they mean:

```wat
:wait (:queue::Queue::Wait::Immediate)                                  ;; was :wait-ns 0
:wait (:queue::Queue::Wait::UpTo (:wat::time::Millisecond 250))         ;; was :wait-ns 250000000
```

**The shape is already proven end-to-end.** `probe-nonzeroduration-crosses-the-wire.wat` is exactly
this: a request-side enum carrying a `NonZeroDuration`, inside a `defsurface`, round-tripping at
**process** locus, with a zero payload refused as `RequestMalformed` while the service stays alive.
It is committed, it is green, and it is the worked reference — not a claim the executor must take on
faith.

### ⛔ NAMES — ruled by the builder after three `intueri` casts. Do not re-derive.

`Poll` / `Block` were considered and **rejected on evidence**:

- **`Poll`** — `circuit.wat:144` says *"Park, don't poll"*, and
  `BRIEF-the-workers-stop-polling.md:3` defines polling as *"`:wait-ns 0` … producing 144,485
  receive calls to deliver 8,000 messages, 136,485 of them empty."* In this tree "poll" is the arc's
  name for **the disease**. Handing it to a legitimate arm is a lie.
- **`Block`** — promises a sleep. `sqs.wat:80` gives `Waiter` a `deadline-ns` field and `:515` builds
  it as `(+ start-ns wait)`: the value is an **upper bound**, and a message arriving at 1 ms returns
  at 1 ms. Believing `Block` costs 250 ms is the adoption failure that already happened once here
  (`a5d696a49`). `Block` also collides with `WouldBlock`, spoken on the *backpressure* axis in the
  stdlib file at `wat/service.wat:2128`.

## ⛔ THE ONE CONTRACT DECISION

**After this stone, no comparison against a wait magnitude may exist anywhere in the queue.**

Not `<= 0`, not `> 0`, not `< 1`. The fork at `sqs.wat:487` becomes a `match` on the enum, and the
mode is read from the **constructor**, never from the number. This is grep-checkable and it is the
whole point: if a magnitude comparison survives, the mode is still being derived from a value and
the stone has changed the spelling without changing the defect.

`Waiter/deadline-ns` stays an i64 — it is a **computed instant**, not a mode, and `(+ start-ns
(nanoseconds d))` is arithmetic on a measurement.

## ⛔ THE CLAMP STAYS — a correction to this stone's earlier draft

An earlier draft said *"delete `sqs.wat:737`'s clamp."* **That was wrong, twice:**

```wat
delay0 (:wat::core::if (:wat::i64::< delay 1000000) 1000000 delay)
```

1. **It is not a zero guard.** The tick's fold (`sqs.wat:678`) keeps only waiters with
   `deadline-ns > now`, so `delay >= 1` always. It is a **tick-rate floor** — it stops a 1 µs
   deadline from arming a 1 µs alarm and ticking the queue a thousand times per millisecond.
2. **It is now a panic boundary.** `arm-tick` (`sqs.wat:211-223`) builds
   `(:wat::service::Alarm :after (:wat::time::Nanosecond delay0))` **from a computed i64**, and after
   Stone A a zero there raises `LociDiedError/Panic`, killing the child at process locus.

It keeps its behaviour and finally gets the WHY comment it never had. The six `1000000` literals at
`:297 :388 :470 :491 :578 :629` are the *former* `Nanosecond 0` sites (`SCORE-the-sane-circuit:53`);
they are now correct values, not workarounds, and are likewise left alone.

## FILES

| file | change |
|---|---|
| `wat-scripts/queue/sqs.wat:44` | the `Queue::Wait` enum, in `:messages` |
| `wat-scripts/queue/sqs.wat:58` | `wait-ns <- i64` → `wait <- :queue::Queue::Wait` |
| `wat-scripts/queue/sqs.wat:443,487` | bind and **`match`** — the `<= 0` fork dies |
| `wat-scripts/queue/sqs.wat:515` | `:deadline-ns (+ start-ns (nanoseconds d))` |
| `wat-scripts/queue/sqs.wat:737` | **clamp stays**, gains its WHY |
| `wat-scripts/queue/sqs.wat:11-12` | the comment is now half-true — **S21**, and this stone owns it |
| `wat-scripts/queue/sqs.wat:783,792,796,861,868` | the three `:user::` helpers |
| 10 call sites in 5 files | **wat-fix codemod** |

**Census, taken across every directory in the repo** (`benches crates docs examples scripts src tests
wat wat-scripts wat-tests workflows`, `target`/`.floor` excluded) — **13 sites, 6 files, all under
`wat-scripts/`**:

| value | sites |
|---|---|
| `0` → `:Immediate` | `sqs.wat:783`, `sns-fanout.wat:490`, `circuit.wat:872`, `circuit.wat:1091`, `probe-visibility-redelivers.wat:45` |
| `250000000` → `:UpTo (Millisecond 250)` | `sns-fanout.wat:217`, `circuit.wat:161`, `probe-three-waiters-wake.wat:63` |
| `50000000` → `:UpTo (Millisecond 50)` | `circuit.wat:343`, `probe-parked-waiters-stop.wat:61` |
| parameter, hand-typed | `sqs.wat:796`, `sqs.wat:868`, `probe-refused-retry-self-consumes.wat:109` |

★ **This census was taken over the whole tree, not a subset.** The last three I took were filtered —
one omitted three constructors, one omitted `wat-tests/` entirely, one reported an empty grep as a
fact. **Do not trust it because I wrote it; the codemod's own finder is the instrument that counts.**

⛔ **This is a `.wat` corpus migration → `wat-fix` codemod, never hand-edits or python/sed.**
Framework `wat/fix.wat`; the nearest recorded shape is `wat-scripts/fixes/response-record-to-enum.wat`
(a record field becoming an enum) and `positional-to-kwargs.wat` (kwarg-value rewriting). Census
first with `wat --grep`, diff it, then apply to every path. `sqs.wat` is **userland, not stdlib**, so
no BOOTSTRAP dance is needed.

## OUT OF SCOPE = REJECTED

- **Collapsing `:user::do-receive` / `do-receive-wait`.** They differ only in this field, and with the
  enum the caller names the mode, so one helper would do. Real, and **not this stone** — it is a
  helper-vocabulary change. **Stone D.**
- **`take-one`, `wait-pending`, `q-depth`, `accept!`.** The lying helper names. **Stone D**, which
  owns the live race.
- **Deleting the clamp.** Rejected above, with the correction recorded.
- **`Alarm :delay`, `Milliseconds`, `visible`/`unacked`.** **Stone C.**

## THE PROOF

1. **★ No magnitude comparison survives.** `grep -n 'wait' wat-scripts/queue/sqs.wat` shows no
   `<= 0`, `> 0`, or `< 1` against a wait. The mode comes from the constructor.
2. **The circuit still holds its invariant.** `distinct=8000; dup=0` — this stone must not change
   throughput materially, and a delta either way is a finding.
3. **A zero cannot be spelled.** `:UpTo (Millisecond 0)` fails to type-check (Stone A, rung 3), and a
   zero arriving over the wire is `RequestMalformed` with the service alive (B-pre, proven).
4. **The floor**, Summary line. `5213/5213`. ⛔ The Stone D race is still live and unfixed — if
   `refused_subscriber_is_retried_not_dropped` goes red, that is the known race firing, and the
   reproducer is the disposition, not the floor.
