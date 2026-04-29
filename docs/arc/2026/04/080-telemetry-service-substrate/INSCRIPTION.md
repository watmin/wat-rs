# Arc 080 — `:wat::std::telemetry::Service<E,G>` — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate now ships a queue-fronted batch-accepting service
shell generic over E (entry type) and G (cadence gate type). Lifted
from the lab's `:trading::rundb::Service` (arc 029) — same shape,
substrate-canonical typename and zero domain knowledge of what an
entry is. The user's load-bearing correction sits at the heart of
the design:

> "the LogEntry /must/ be user defined - we do not provide anything
> here.. or maybe we provide extremely basic things to educate the
> reader on how to implement their own bespoke entries with
> whatever complexity"

The substrate ships the SHELL (queue, driver, cadence, stats,
typealiases, batch-log client primitive); the consumer brings the
entry enum, the dispatcher, and the stats-translator. Substrate
discovers nothing about the consumer's data shape; consumers
discover nothing about the substrate's queue plumbing.

**Design:** [`DESIGN.md`](./DESIGN.md).

---

## Naming correction from DESIGN to as-shipped

DESIGN.md proposed `:wat::std::telemetry::Sqlite<E,G>` thinking the
service would be sqlite-flavored. **It isn't** — the service is
generic over destination too; the dispatcher closure is what knows
where each entry lands. Renamed to
`:wat::std::telemetry::Service<E,G>` (file: `wat/std/telemetry/Service.wat`).
Sqlite + Console are sibling DESTINATIONS that compose with this
generic shell — see arcs 081 (Console dispatcher factory) + 083
(Sqlite/spawn delegate).

---

## What shipped

### File: `wat/std/telemetry/Service.wat` (~290 lines)

Eleven elements following arc 078's service contract:

1. **`Stats` struct** — substrate-defined counter set
   (`batches`, `entries`, `max-batch-size :i64`).
2. **`MetricsCadence<G>` struct** — same shape as arc 078.
   `(gate :G) (tick :fn(G,Stats)->(G,bool))`.
3. **`null-metrics-cadence`** — opt-out factory whose tick never fires.
4. **`Stats/zero`** — fresh zero-counters factory used at startup +
   on each gate-fire.
5. **Protocol typealiases** — `AckTx` / `AckRx` / `AckChannel` /
   `Request<E>` / `ReqTx<E>` / `ReqRx<E>` / `ReqChannel<E>` /
   `ReqTxPool<E>` / `Spawn<E>` / `Step<G>`.
6. **`Service/tick-window<E,G>`** — gate-fire logic. Always advances
   cadence; conditionally builds `Vec<E>` via translator and
   dispatches each through the SAME closure that handles client
   batches.
7. **`Service/loop<E,G>`** — recursive select loop with confirmed
   batch + ack + heartbeat. Per-iteration: select → dispatch batch →
   ack client → update Stats → tick window → recurse.
8. **`Service/batch-log<E>`** — client primitive (single
   confirmed-batch verb per arc 029 Q10 discipline).
9. **`Service/run<E,G>`** — worker entry. Wraps loop with initial
   Stats.
10. **`Service/spawn<E,G>`** — caller-side wiring. N pairs +
    HandlePool + spawn the worker.

### Tests: `wat-tests/std/telemetry/Service.wat`

3 deftests:
- `test-spawn-drop-join` — lifecycle without traffic.
- `test-batch-log-dispatches` — three batches dispatched through a
  collector closure; assert all 5 entries arrive in order.
- `test-cadence-fires-emit-self-stats` — counter cadence fires every
  3 batches; stats-translator produces 3 self-Stats entries; collector
  sees them mixed with client batches at the right boundaries.

All 3 green.

---

## What changed from DESIGN

- **Naming**: `Sqlite<E,G>` → `Service<E,G>`. Generic over destination,
  not sqlite-flavored.
- **Substrate ships ZERO entry variants.** DESIGN's "tiny educational
  enum" lives in test files only, not in baked stdlib. Per memory
  `feedback_no_speculation`, the educational example is a test, not a
  shipped surface.
- **No companion `wat-telemetry` sibling crate.** The Service shell
  is pure-wat composition over already-baked primitives (kernel,
  HandlePool); no Rust shim needed. Lives in the substrate's
  `wat/std/telemetry/` tree alongside the existing Console primitive.

## What's still uncovered

- **Backpressure isolation between cadence-fire and client batches.**
  Today's loop runs both through the same dispatcher serially. If a
  consumer's dispatcher is slow, cadence emissions queue behind
  client work. Future arc could add an opt-in "dispatch heartbeat
  asynchronously" knob if profiling shows it matters.
- **Per-message routing ack channels.** Each batch carries its own
  ack-tx; that's by design (arc 029 Q10). A future arc could fan-in
  multiple ack channels onto one shared response channel if a
  consumer wants different shape — currently no consumer needs it.

## Consumer impact

Unblocks:
- **Arc 081** — Console dispatcher factory composes with this
  Service shell.
- **Arc 083** — Sqlite/spawn was the lab's first consumer; it
  delegates to Service/loop with hooks for schema-install + dispatcher.
- **Arc 085** — `Sqlite/auto-spawn` derives the dispatcher from an
  enum decl; wires through the same Service shell.
- **Future cross-domain destinations** (file appender, network sink,
  whatever) — same generic shell, different dispatcher.

PERSEVERARE.
