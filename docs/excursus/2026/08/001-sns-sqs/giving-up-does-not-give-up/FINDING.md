# FINDING — `GaveUp` bounds the wait, not the work

> ⛔⛔ **CORRECTED SAME DAY — THIS WAS A CATEGORY ERROR, AND THE SUBSTRATE IS RIGHT.**
> Builder: *"this problem does not exist for threads — they are a completely different beast. The
> partition line is 'do you use shared memory or not?' The IPC is the first non-shared memory; the
> remotes when we create them (loopback tcp, loopback mtls, loopback dtls, actual remote hosts)
> will be the next non-shared memory locus. Threads don't have this problem as they are in a
> completely different fault domain."*
>
> **Measured on BOTH loci, same bound, same fixture, `WAT_COLLECT_DEADLINE_MS=200`:**
>
> | locus | nap 1000 | nap 10000 | nap 30000 |
> |---|---:|---:|---:|
> | **process** — first non-shared memory | **387 ms** | **343 ms** | **362 ms** |
> | **thread** — shared memory | 1136 ms | 10144 ms | (30126 ms) |
>
> ⭐ **The process tier is already bounded.** Wall time is flat at ~350–390 ms across a 30× spread
> of abandoned work: the coordinator gives up at 200 ms and exits. The child is a separate fault
> domain and is reaped.
>
> **So the original framing below is wrong on the point that matters.** The thread numbers are not
> a DoS and not a defect: a thread worker shares the coordinator's address space and fault domain,
> so *"my work-fn naps 30 s and my process takes 30 s"* is the program the author wrote, not a
> worker holding a coordinator hostage. **No trust boundary is crossed inside shared memory.** The
> S2b drain-then-join analysis below is still an accurate description of the thread-tier mechanism
> — it is just not evidence of a hazard.
>
> ⚠ **What survives:** the partition itself, now measured rather than asserted —
> **shared memory / not** is the line, IPC is the first crossing, and the bound is real exactly
> where a separate fault domain exists. That is a property to *hold* as the remote loci arrive
> (loopback TCP, mTLS, DTLS, real hosts), not a bug to fix.
>
> ⚠ **And a method failure of mine, recorded:** my first process-tier runs at nap 1000/10000/30000
> appeared to track the nap — because I dropped `WAT_COLLECT_DEADLINE_MS` from those invocations,
> so the 300 s default applied and the bracket never gave up. The table above is the corrected run.
> A measurement whose knob is not armed measures the default.

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
