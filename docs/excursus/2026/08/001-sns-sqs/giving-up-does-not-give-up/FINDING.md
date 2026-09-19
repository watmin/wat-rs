# FINDING — `GaveUp` bounds the wait, not the work

**Measured 2026-09-19, HEAD `923732e27`** (immediately after `select-by-deadline` landed). Census
and diagnosis only; nothing changed.

## The measurement

`probe_every_worker_races_n_stall.wat` with `WAT_COLLECT_DEADLINE_MS=200`, a pool of 3, work-fn
naps N ms:

| nap | bound | `GaveUp waited-ms` | **process wall** |
|---:|---:|---:|---:|
| 500 | 200 | 200 | **638** |
| 1000 | 200 | 201 | **1123** |
| 3000 | 200 | 200 | **3161** |
| 30000 | 200 | 200 | **30126** |

⭐ **The bound fires correctly every time. The process wall tracks the NAP, not the bound.**

The coordinator gives up on schedule, raises `GaveUp`, and the process is then held open for the
full duration of the work it just abandoned. At 30 s that is a 150× overrun of a 200 ms deadline.

⚠ This was already visible in a **green** test and nobody read it: `n_stall_gives_up_not_hang`
takes 2.17 s with a 2000 ms nap and a 200 ms bound. The 1.97 s gap is the process waiting for
runners the coordinator had abandoned.

## The mechanism, and it is DELIBERATE

`src/kernel/spawn.rs` — arc 259 S2b:

> *"Dropping the peer value must, via the peer's RAII `Drop`, **drain** (drop the input Sender →
> the worker's `recv` raises → the worker exits) then **join**. Because `join` is synchronous, by
> the time `drop` returns the worker has fully exited… at HEAD the peer's `JoinHandle` detaches and
> the worker is reaped asynchronously (**the detach race S2b eliminates**)."*

So:

1. `GaveUp` raises; the peers drop.
2. `Drop` **drains** — the input sender goes, so the worker's **next `recv`** will raise.
3. ⛔ **A worker that is WORKING is not in `recv`.** The drain cannot reach it.
4. `Drop` **joins** — and blocks until the work finishes and the loop returns to `recv`.

⭐ **The drain reaches a WAITING worker. It cannot reach a BUSY one.** That is the whole finding,
and the join is not a bug — it was added on purpose to make reaping deterministic.

## Why this matters, in the builder's terms

> *"it can be dos'd by its worker fleet with IPC."*

Confirmed a second way, and this one survives the deadline. A worker with a long task holds the
coordinator's **process** open for that task's full duration **after** the coordinator has given up.
The wall-clock bound the whole `every-worker-races-its-own-timer` stone exists to provide is
honoured by `collect-loop` and **ignored by process exit**.

And it is the mechanical form of the builder's own instinct:

> *"we also need a 'RST wake' or something to kill the timedout worker."*

This is the evidence that it is needed. Without it, "give up" means "stop waiting, then wait anyway".

## ⛔ The hard part, named rather than deferred

**A thread cannot be safely killed in Rust.** There is no `Thread::kill`. So bounding abandoned
work at the thread tier needs one of:

| option | cost |
|---|---|
| **cooperative cancellation** — the worker polls a stop flag | a wat work-fn is opaque user code; it will not poll, and a napping one cannot |
| **detach instead of join** | ⛔ reintroduces exactly the race arc 259 S2b was built to eliminate |
| **process-tier runners, signalled** | works — a process can be killed — but is not available at the thread tier |

⭐ **So this may be a tier-shaped limit, not a bug to fix.** *"A thread peer cannot have latency, a
full socket buffer, or a partition"* — and it may also be that a thread worker cannot be abandoned.
If so, that is an argument for process/remote runners whenever a bound must be real, and it belongs
in the brackets-are-networked story rather than being patched at the thread tier.

⚠ **Do not "fix" this by detaching.** The race S2b removed is a real one and the SCORE for any such
stone must face it directly.

## What this does NOT claim

- Not that `select-by-deadline` is wrong — it does exactly what it says, measured above.
- Not that the process-tier behaves the same way; **unmeasured**, and it is the first thing a stone
  here should measure, since a process runner *can* be killed.
- Not that the existing tests are wrong — they assert what they assert. One of them has been
  showing this for hours and was read as "a bit slow".
