# DESIGN — fill deep, then drain

**The queue reaches a known depth before anything consumes it, and the drain is timed alone.**
`wat-scripts/fanout/circuit.wat` only.

## WHY — "351 pairs/sec" was never measured at a known depth

The tracker's headline number comes from one run with `cap 16384` hand-edited in. In that run the
subscriber workers were **already consuming while the publishers published** — as they are in every
run today — so the depth at which the drain began is unknown, and the rate is a steady-state
number from an overlapped regime, not a drain rate.

The builder's question is the SQS one: **fill to a depth, then measure how fast it empties.** That
is a different experiment and the harness has never run it.

## ⛔ WHAT THE READ FOUND — the deferral already exists

Workers are **spawned idle and armed by an explicit signal**. `:demo::start-topic-worker!`
(`sns-fanout.wat:678`) sends a `StartRequest` that arms the tick; `:fanout::start-worker!` does the
same for the subscriber workers. The circuit issues both **before** the publish timer:

```
:1910  _twgo   arm the j topic-workers
:1969  _go     arm the m×j subscriber workers
:1974  t-pub0  ← publish timer starts here
:1990  pub-pair = join-publishers   (publishers done)
:1995  t-drain0
```

★ So "fill first" is a **line move**, not a build. Moving `_go` past `:1990` leaves the topic
workers fanning out into the subscriber queues while nothing drains them.

## ⛔ THE ONE CONTRACT DECISION — three knobs, defaults preserve today

`run-with` gains **`sub-cap`** (the subscriber queues' `:cap`, hardcoded `32` at `:1837`) and
**`fill-first?`**. `:user::main` gains an argv path.

```
(:wat::runtime::argv)  →  no args : today's run* 2000 4 3, unchanged
                          args    : <n> <m> <j> <sub-cap> <fill-first?>
```

`argv` index 2 is the first user argument — verified this session against
`wat-scripts/scratch-pad/255-stone-o-iv-c-1-span-still-points-at-caller.wat`, which reads
`(:wat::core::get argv 2)` and whose usage `Option/expect` fires with no args.

★ **One point per invocation, shell-driven.** The sweep is a bash loop, not a wat loop: a crash at
depth 8 does not lose depths 1–7, and each point is an independent process with no carried state.

### The phase split

With `fill-first?` the existing boundaries almost work already — `pub` ends at `join-publishers`,
which is exactly the end of the fill. One phase must be **added**, not reused:

```
setup   wiring                          (unchanged)
fill    publishers → topic → sub queues, NOTHING consuming
arm     ← NEW: arming m×j workers is IPC, and it is not draining
drain   poll until empty, from a known depth
```

⚠ Arming 12 workers is 12 process round trips. Folding that into `drain` would put the
instrument inside the measurement — the error this arc has already made three times. **A phase
measures one thing or it measures nothing.**

## ⛔ THE ROW THAT IS THE STONE — prove the fill actually filled

The measurement is worthless unless the depth at the fill/drain boundary is **observed**, not
assumed. Between `join-publishers` and the arm, take one sweep and report it:

```
fill-depth=[2000/0][2000/0][2000/0][2000/0]
```

★★ `sweep-of` — built by the previous stone — is exactly this, and `qclients` already exist at
that point (`:1883`). If the reported depth is not `n` per queue, the fill did not fill and every
rate derived from the run is void. **This is "publish was never publish" turned into a gate.**

## ⛔ THE LIVENESS BOUND MUST BE DERIVED, NOT RAISED

`poll-until-drained qclients topic 4000` (`:1996`). At 4000 attempts × 5 ms that bound is ~20 s of
polling; a deep drain will exceed it and raise `drained-never`.

⚠ The bound carries an explicit ruling — *"only a hang may trip this … a red here is STUCK, never
'the box was busy'"*. It may **not** be hand-raised to fit a longer run. It must become a
**function of the work**: attempts derived from `n × m`, so the bound scales with what is being
drained and still catches a hang at any depth. A hand-picked larger constant is the thing the
ruling forbids.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

⚠ **`sub-cap` is not a contract limit.** `:cap` is `:durable` per-instance state on
`queue::queue::Record` (`sqs.wat:116`) — configuration chosen by whoever starts the instance. The
wire contract is `:max-entries [bodies 64]` / `:max-page [envelopes 64]` (`sqs.wat:102,104`), and
**those do not move.** Raising `sub-cap` is the harness configuring an instance it owns; it is not
the instinct-to-raise-a-limit this arc has already ruled against.

★ The answer is not predicted here. If pairs/sec is **flat** across depth, the system paces
honestly under load and the queue-bound finding is a constant. If it **degrades**, that is the
thing to attack and it will be a real property rather than our own instrumentation — which is what
`a count never reads more than it needs` and `the poller sweeps once` were drawn to guarantee.

## OUT OF SCOPE — REJECTED

- **The inbox `:cap 64`** (`:1847`). Topic-workers run during the fill, so the inbox stays shallow.
  Not the queue under measurement; leave it.
- **`collect` 5.9 s.** A separate open item — `collect-stop` ships 8000 Outcome records. It will
  dominate the total at depth and must be **reported and ignored**, not fixed here.
- **Tuning the 5 ms poll interval.** A second variable would make the depth curve unreadable.
- **A wat-level sweep loop.** The shell is the loop; `main` runs one point.
- **Deferring `_twgo` as well.** That would pool depth in the inbox instead of the subscriber
  queues, which is a different experiment (and the inbox is one queue, not m).
