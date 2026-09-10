# DESIGN — one question per worker

## Why

Builder: *"i thought we already agreed to do the 'don't ask two questions' thing."* Correct — I named it
three times across `1196720c3`, `c833f4a6e` and the `collect` review, and never drew it.

**`collect` is 5141 ms and only 4 % of it is work** (`collect-busy-ms` 208). Two-thirds is per-worker
round-trip latency, and the mechanism is confirmed both directions at `c833f4a6e`:

```
:fanout::worker's receive wait   collect (12 workers)      per worker
        50 ms                     962 ms                     80 ms
       250 ms  (shipped)         4053 ms                    338 ms
       500 ms                    7695 ms                    641 ms
```

**A request to a worker waits out the worker's own blocking `Queue/receive`.** `collect` asks each of 12
workers **two** questions — `sum-disrupts` (`circuit.wat:2533`) and `collect-stop` — so it pays that wait
**twice per worker**, which is exactly the measured ~1.3× the poll interval.

★ And under the networking-first ruling this is not a millisecond question: it is **12 avoidable
crossings**, each of which is ~1.0 ms of bare IPC on this box and a full RTT off it.

## What it delivers

**The worker's disrupt counters ride the `:stop` projection**, so `collect` asks each worker once instead
of twice. `sum-disrupts` disappears.

Expected: `collect`'s per-worker term halves — ~2400 ms at the shipped 250 ms poll — and 12 crossings go.

## ⭑ The channel already exists and is already author-chosen

`wat/service.wat:2928-2968`: the generated `stop` is one `send Admin::Stop` + one `recv`, returning a
**`:stop` projection of the author's own type**. `wat-tests/service-stop-resp.wat` is the worked exemplar —
a service whose `:stop` renders final state to an `i64`.

★★ So this is **not** new substrate. It is using an affordance that exists, on a round-trip the harness
already makes. **Zero new crossings, and one fewer per worker.**

## The one contract decision

⛔ **The stop projection must carry the counters WITHOUT the harness losing the ability to read them on a
live worker.** Today `sum-disrupts` can be called at any time; after this it can only be read at stop.

**That is acceptable here and the reason must be recorded:** `collect` is the only caller, and it runs
after the drain, immediately before stopping the workers. **If any future caller needs disrupts from a
live worker, this stone has taken that away** — so the DESIGN says so out loud rather than discovering it
later.

⚠ And a live-read is *not* free to keep: it costs a crossing **and** waits out the worker's poll. Keeping
it "just in case" would preserve exactly the cost being removed.

## Out of scope = rejected

- **Making a parked worker answerable** — the real long-poll fix. `a4f2d7f7b` established the gap
  precisely: **outbound YES** (`kernel::send` in a handler returns immediately), **inbound NO** (a peer's
  reply cannot arrive as a service op), and closing it needs a fifth `asks` field on the outcome, a third
  superset source, and a select set admitting a dialed peer — **all reaching `src/`.** That is an arc and
  the builder's to open. **This stone is the cheap half that needs none of it.**
- **`collect`'s remaining third** — the record fold and `empty-flags`. Measured at 36–112 ms; not worth a
  stone.
- The maintained-depth patch (blocked on `DeleteResponse::Success` being nullary), `fill`'s `p=1` axis,
  the give-up class at four sites.

## ⚠ What must not be claimed

⚠ **This does not make a worker answerable while parked.** It reduces how often we ask, not what it costs
to ask. The 150 ms-per-question cost stands; we simply stop paying it twice.
⚠ And **`collect` is ~76 % process-lifecycle and observability, not message work.** Halving one term of it
does not make the *system* faster — `distinct`/`dup` and the drain are untouched, and a SCORE claiming
throughput has measured the wrong thing.
