# DESIGN STONE — the buffered span (arc 278 item (c))

**Commissioned 2026-08-31.** Resumes item (c), which `DESIGN-service-io-budgets.md` has named as the
resume point since 2026-07-21 and which that file records as **"NOT built (nothing on the disk)."**

## What item (c) asked for, and what it should be instead

The original sketch (22 lines) proposed `with-log-sink` — a **bracket**-managed sink, "ephemeral
lifetime, RAII". **That framing is wrong and is not being built.** `wat/bracket.wat` is a worker
pool: `map`/`each`/`map-worker`/`PoolMsg`/`collect-loop`, fan-out over items and fan-in of results.
A log sink fans out over nothing. The sketch reached for bracket because it wanted RAII, not because
it wanted a pool.

**There is no new service and no new surface.** `span'` is already the sink:

- it is already a `defservice` held by the producer's scattered call points;
- it already accumulates pure state — `counters`, `durations`;
- it already flushes accumulated state to the Journal — `close`;
- it already holds the sink peer where a live resource must live — `:ephemeral [sink <- Peer]`,
  because `:durable` is *"the soul: EDN, crosses the wire"* and a peer cannot cross a wire.

`log` is the single arm that breaks the pattern. Today it builds one `Log` and ships it immediately:

```wat
(:wat::telemetry::Journal/write-logs (…/sink s)
  (…::WriteLogsRequest (:wat::core::Vector :- [:wat::telemetry::Log] l)))
```

**A batch of one, per line.** The batch API has been there all along, handed one item at a time.

★ **Why an actor and not a client-side buffer:** the producer logs from *scattered call points*.
Sharing one buffer across them is shared mutable state, and wat has exactly one sanctioned answer —
`wat/query/mem.wat`: *"the actor's loop is the ONE place mutation 'happens' (by rebinding, not by
mutating memory)."* The hop is already being paid; batching removes the expensive one.

## The shape

```
:durable [… counters … durations …
          logs <- (:wat::core::Vector :- [:wat::telemetry::Log])]   ;; one more accumulator

log     → conj into `logs`      (exactly what `incr` does to `counters`)
flush   → ship + RESET          (exactly what `close` does, minus the finality)
close   → flush the REMAINDER
```

`ctx` already carries the identity every emission needs — `namespace`, `invocation-id`, `start-ns` —
and was built for it: a `SelfInvocation` *"still gets a ctx because it is still an INVOCATION: a
thing the service DID, at a time, which must be visible to telemetry exactly like a client call."*
So a timer-fired flush is attributable, by design, before the timer exists.

## ★ THE CONTRACT DECISION: flushes emit DELTAS, and `close` flushes the remainder

`close` today emits **one Metric per counter, from the accumulated total**, and it is the only
emission. Add any second emission that also sends totals and a 90-second span sends
`requests=100`, `250`, `400`, then `close` sends `400` again — and any backend that sums counters
(CloudWatch, statsd) reads **1150**.

So:

> A flush emits the counts **accumulated since the last flush** and **resets** them. `close` stops
> meaning "emit everything" and becomes **"flush the remainder."**

Periodic and final flush then become the *same code path with the same reset*, which is the point:
one emission story, not two that must be kept consistent. **A design where `close` and a periodic
flush emit differently is rejected** — that is the double-count, structurally.

## Durations emit BOTH — aggregate and fidelity

`Samples` is already `(Vector :- [i64])`: the span holds **full fidelity today** and `close` throws
it away, emitting only count and sum. Both are worth having and they answer different questions:

| emitted | name | unit | answers |
|---|---|---|---|
| count | `<name>/count` | `Count` | rate |
| sum | `<name>/duration` | `Nanos` | mean, total time |
| **each sample** | `<name>/sample` | `Nanos` | **percentiles, the tail** |

`<name>/count` and `<name>/duration` are the existing convention (`span.wat` close) and do not
change. `<name>/sample` is additive.

The asymmetry decides it: **count+sum is lossy and unrecoverable** — store `[142, 8_400_000]` and
p99 is gone forever, and a mean cannot distinguish "every call took 60µs" from "one call took 5ms".
Fidelity can always be reduced at read time, and the read side already exists: `sift-metrics`
compiles a `Sieve` once and applies it per row. Percentiles are a fold over sifted rows.

Cost, stated: `Numeric` is a scalar, so fidelity is **one Metric per sample** — N rows, not one row
holding a vector. The SORTKEY stone gave each event its own identity, so N distinct rows is sound.
If volume ever forces a cut, cut fidelity — count+sum is what exists today and survives.

## Two triggers, and only one of them is a timer

- **Size** — an `if` in the `log` arm after the `conj`. The threshold is **derived from the
  contract**, not a magic number: `write-logs` declares `:max-request-bytes 10485760`, and the
  io-budgets arc made per-op budgets declared on the surface and *discoverable*. Same for durations.
- **Time** — a self-scheduled alarm. Supported by construction: `Outcome::NoReplyAndArm` takes
  `arms <- (Vector :- [Alarm])` and each `Alarm` carries **its own `after` AND its own `op`**, so
  two internal ops re-arm at independent cadences (`-flush-logs` fast, `-flush-counters` at 30s).

## The stones

| stone | what | why this order |
|---|---|---|
| **A** | the accumulator, the delta/reset contract, both duration emissions, size trigger | the contract decision, provable with NO timer |
| **B** | two internal ops at independent cadences | pure scheduling on top of A's flush path |

Stone A first because the double-count is the only thing here that can be *silently* wrong, and it
is fully testable without a clock. A timer would make that test time-dependent for no benefit.

Out of scope = REJECTED: `with-log-sink` as a bracket; any new surface; any change to `Journal`;
any change to `Numeric` to hold a vector.
