# DESIGN — every client call has a deadline

**The unguarded round-trips.** `wat/service.wat` + `wat-scripts/fanout/circuit.wat`.
Correctness. No perf work.

## WHY — T1 deadlined one call out of four, and the pattern never generalised

The worker's client calls, censused:

| call | guard | |
|---|---|---|
| `Seen/check` | `select [peer tmr]`, 200 ms, hand-rolled | ✅ |
| `Seen/mark` (`circuit.wat:515`) | none | ⛔ |
| `Queue/ack` (`:526`) | none | ⛔ |
| `Queue/receive` (`:402`) | `:wait (UpTo 250 ms)` | ⛔ **not a deadline** |

★ **`:wait` bounds the SERVER's long-poll; a deadline bounds the CLIENT's wait for an answer.**
A dropped reply defeats the first entirely and is caught only by the second. The code relies on
`:wait` as if it were a deadline; it is not.

`(_ nil)` on the bare calls catches `Lost`/`Closed`/`Stopped`, so a **dead** peer returns. A
**silent** peer — reply dropped, connection alive — blocks forever. The previous stone's
executor hit exactly this: dropping `mark` hung a worker ~160 s, and the workaround was to stop
dropping `mark` in the harness. **The workaround lives in the chaos injector; the exposure lives
in the worker.**

## ⛔ WHY THIS IS A SHARED HELPER AND NOT THREE PATCHES

`check` does not use its generated method at all. It drops to
`(:wat::kernel::send peer (:fanout::Seen::Op::Check req))` plus select, timer, redial and a
three-way retry — **~40 lines, inline.** The undeadlined path is a one-liner.

★ **The wrong path is the easy path.** That asymmetry is the defect; three copies of the 40-line
dance would entrench it. `extirpare`: make the right path the easiest path.

## ⛔ WHERE IT CAN LIVE — measured, not assumed

`probe-what-a-process-impl-can-call.wat`, committed first. The same service at both loci:

```
THREAD  -> the impl calls a plain sibling defn and replies
PROCESS -> the child dies at startup: UnknownCallee :pc::plain-helper
```

| candidate home | reachable from a process impl? |
|---|---|
| a plain `defn` in `circuit.wat` | ⛔ **NO** — this probe proves it |
| a `defsurface`-generated method | ✅ (the worker calls `Seen/mark` today) |
| a stdlib `defn` in `wat/` | ✅ (sqs.wat's impls call `:wat::edn::write` today) |

**So the helper goes in `wat/service.wat`, beside `:wat::service::send-keep-serving?`** — which
is already a parametric stdlib helper reachable from every service, and is the precedent to copy.

⚠ It must be added **after** the `defservice` macro, as `send-keep-serving?` was, so nothing
above `wat/service.wat:896` moves. (The bijection goldens snapshot that span. Their real cost was
measured on 2026-09-05 — eight integers and one `UPDATE_EDN=1` — so this is tidiness, not a wall.)

## ⛔ THE ONE CONTRACT DECISION

**The helper returns whether an answer arrived, and the caller cannot confuse a timeout with a
reply.** `select` reports which selectable fired; `idx` is the discriminator, and the timer's
payload is **inert** — required by the type, never read. The existing `check` code already works
this way and that property must be preserved, not re-derived per call site.

Shape (parametric, like the seam):

```wat
(:wat::core::defn :wat::service::call-by-deadline :- [I O]
  [peer <- (:wat::kernel::Peer :- [:I :O])  op <- :I
   ms <- :wat::core::i64  inert <- :O]
  -> (:wat::core::Tuple :- [… (:wat::core::Option :- [:O])]))
```

`None` means **the deadline fired**, not "the server said no". A caller that wants a retry
redials and calls again — the same discipline T1 already proved on `check`.

## FILES

`wat/service.wat` (one new parametric defn at the end) and `wat-scripts/fanout/circuit.wat`
(four call sites: `check` refactored onto the helper, `mark`/`ack`/`receive` gaining one).

## OUT OF SCOPE = REJECTED

- **Making an undeadlined generated client method unrepresentable.** That is rung 3 and the right
  eventual answer — every `defsurface` method would take a deadline — but it is a surface change
  across every service in the tree and needs its own stone and its own census.
- **All perf work**, including the send-path double scan and the two seen round-trips.
- **The `claim deadline exhausted` crash** and **the fixture that lost its meaning** (both open
  from the previous SCORE). Named, not touched.
