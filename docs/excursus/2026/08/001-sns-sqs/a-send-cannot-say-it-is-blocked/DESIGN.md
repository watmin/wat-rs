# DESIGN — a send cannot say it is blocked

**Drawn 2026-09-15. NOT STRUCK, and it must not be struck without a builder ruling** (§THE RULING
OWED). Sibling of `a-wait-that-should-be-bounded` / `both-ends-bound-the-same-handshake`, which closed
the `recv` half of the builder's invariant: *"both sides needs ot be able to time out and recover"*,
*"no one is allowed to crash on a recoverable error."*

## Why now: the recv half is CLOSED, so `send` is the surface

`a-wait-that-should-be-bounded/FINDING-the-classification.md` read all 23 live bare-`recv` sites in
context: **4 BOUND** (all four struck — `7686bea24`, `ab419aaa3`), **11 TIMER**, **7 PARK**, **1
UNKNOWN** (`recv-all-loop`, whose bound is a return-contract ruling). **Zero BOUND sites remain.**

The waiter census re-run at `ab419aaa3` gives the next surface:

```
send yes=0  no=215  UNKNOWN=0        ← corpus-wide
live (wat + wat-scripts/service) send sites = 32
try-send yes=5                        ← the ONLY send-side bound in the corpus, and its bound is ZERO
```

## ⭐ THE FINDING THAT MAKES THIS DIFFERENT FROM THE RECV HALF — MEASURED, NOT ARGUED

`wat-scripts/scratch-pad/probe-a-full-inbox-blocks-send-forever.wat`, process tier:

| t | output |
|---|---|
| ≈2 s | `full-at=283` — the inbox is durably full |
| 5 / 10 / 20 s | still one line — **the plain `send` is BLOCKED**, ≥ 38 s |
| at the kill | `plain-send=Stopped` — it returned **only** because `timeout`'s SIGTERM killed the service |

⛔ **`SendOutcome` is `Sent | Closed | Lost | Stopped`. No variant describes "blocked / not draining".**
Every recv site the last two stones touched already had a `TimedOut` arm written and unreachable — arc
109's *painted brick*, `NOTE-an-outcome-variant-no-primitive-can-construct.md`. **The send side has no
arm at all.** The value the blocked site finally observed names the peer's **death**, so a caller cannot
distinguish *"I blocked 38 s and then it died"* from *"it was already stopped"*. That is not a usage gap
the way the recv asymmetry was; it is a **missing form**.

⚠ **And the first fixture refuted itself — record it, because it will mislead the next reader too.**
Fixture 1 reused `probe-crash-surface-try-send-wouldblock.wat`'s service (a handler that returns
immediately without replying). `try-send` reported `WouldBlock` at **n=376** and a plain `send`
immediately after returned **`Sent`**, in 1 s total. The reason: a `defservice` serve loop **does**
drain — it recvs every Op and dispatches; what that handler never did was *reply*.
⭑ **`TrySendOutcome::WouldBlock` does NOT imply the receiver has stopped draining.** It can be
transient socket pressure. Only a handler that **parks** (fixture 2) makes the inbox durably full.

⚠ One asymmetry in the *other* direction, kept because it bounds how bad this is: the blocked `send`
**did** return when its peer died, where a blocked bare `recv` ignored SIGTERM for 125 s.

## ⛔⛔ THE RULING OWED — three mechanisms, and none is a code-reading

A bounded `send` **does not exist in any form**. The corpus offers no shape to copy, which is why this
DESIGN stops here instead of naming an implementation:

| # | mechanism | cost |
|---|---|---|
| a | **retry-loop around `try-send` + wall clock** (wat only, no `src/`) | busy-waits; and `WouldBlock` is proven NOT to mean "not draining", so the loop's exit condition is measuring the wrong thing |
| b | **`send-by-deadline` primitive** + a new `SendOutcome::TimedOut` | reaches `src/`; a new variant on `SendOutcome` forces an arm at **every** send match in the corpus — 215 sites |
| c | **`SendByDeadlineOutcome`**, a separate enum, leaving `SendOutcome` alone | no churn on the 215; two enums for one operation, which is the drift `both-ends-bound-the-same-handshake` was struck against |

⭑ (b) is the honest shape and the expensive one; (c) is cheap and duplicates; (a) needs no ruling and
may not work. **This is the builder's call**, exactly as `RecvOutcome::Stopped` (325 arms) is.

## What this stone IS, if struck as drawn: a classification, REPORT-ONLY

The mirror of `a-wait-that-should-be-bounded`: read all **32 live** `send` sites in context and class
each **BOUND** (a blocked send would hang something that should report) / **PARK** (blocking is the
loop) / **REPLY** (a serve-loop answer — dropping it is worse than blocking) / **UNKNOWN**. No `.wat`,
no `.rs`. The classification is what makes the ruling above cheap to take; taking it first would be
choosing a mechanism for sites nobody has read.

```
wat/service.wat      17   the generated serve loop + client methods + call-by-deadline's own send
wat/bracket.wat       8   map/reduce runners and collect-loop
wat/spawn.wat         2   both ends of the handshake just bounded on the recv side
wat/test.wat          1   run-thread
wat-scripts/…         4   circuit 2 (deadline-redial-is-fresh) · sqs 2 (park-receive!)
```

⚠ `wat/spawn.wat:527` and `:597` are the **send halves of the very handshake `ab419aaa3` bounded on the
recv side** — `launch` sends the ship, then waits. Bounding the wait and not the ship is half a fence.
Start there.

## Trap-doors named up front

1. ⛔ **Do not reuse fixture 1.** `WouldBlock` without a parked handler is transient; a stone that
   builds on it will measure nothing and report a bound that isn't.
2. ⛔ **Do not pipe a blocking probe to `tail`/`head`.** It hid the whole first measurement: no output
   is indistinguishable from a block. Redirect to a file and read it WHILE IT RUNS.
3. **`try-send`'s 5 `yes` rows are not 5 bounded sends** — 4 are scratch-pad probes; the only live one
   is `wat/service.wat:2581`, the serve loop's reply.
4. **Count with the census, not by hand.** `wat-scripts/census-waiter-bounds.wat` prints bounded sites
   by name; re-run it rather than adjusting its numbers (it moved 679 → 681 this session).
5. **A `send` that blocks is not a crash.** It is the same class as the recv work — unkillable-by-SIGTERM
   hang — and the invariant that covers it is the builder's *"both sides"*, not a crash census.
