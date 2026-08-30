# `wat-scripts/` — live wat programs (fixes, demos, pipeline)

> 2026-08-30 revival (wat-revival): the ten mothballed `*.wat.disabled`
> programs from ~arc 103 were either re-enabled under the
> `every_wat_scripts_file_loads` gate or deleted with a reason. Deleted
> (API gone / capability-walled — not a syntax pass): `count-logs`,
> `metrics-summary`, `seed-fixture` (telemetry sqlite stream/auto-spawn
> removed), `ping-pong`, `ping-pong-fork`, `dispatch` (`spawn-program`
> restricted to `:wat::spawn::`/`:wat::test::`; `fork-program-ast`
> retired). Revived: the Unix-pipe trio `router` / `aggregator` / `sink`
> plus `pong`.
>
> `demos/aggregates/showcase.wat.disabled` is not one of the ten; left
> mothballed. `perf/deep-cascade.wat` is kept `.wat` on purpose (broken,
> but in active rework). The `fixes/*.wat` codemods remain the
> up-to-date exemplars for current syntax.

The scripts run against the bundled batteries-included `wat`
binary — no per-script build step needed. A consumer treats wat-rs
like ruby for the duration of this session.

## Scripts

### Deleted — telemetry interrogation (arc 093)

`seed-fixture.wat`, `count-logs.wat`, `metrics-summary.wat` — deleted
2026-08-30. They called `:wat::telemetry::sqlite/stream-logs`,
`Sqlite/auto-spawn`, and `null-metrics-cadence`, which no longer
exist. `:wat::sqlite::open-readonly` remains, but there is no
stream combinator over Event::Log rows. Reviving them needs a
substrate API that was removed (STOP-2).

### Deleted — hologram spawn (arc 103a/103c)

`ping-pong.wat`, `ping-pong-fork.wat`, `dispatch.wat` — deleted
2026-08-30. `:wat::kernel::spawn-program` is restricted to
`[:wat::spawn:: :wat::test::]` and its signature is now
`(locus prog)`, not `(src None)`. `:wat::kernel::fork-program-ast`
is retired (`BareLegacyForkProgram`). A `:user::`/`:demo::`
application cannot spawn without a substrate change (STOP-2).
`pong.wat` survived as a pipe stage (no spawn).

### Pipeline composition (arc 103a) — live

A four-stage Unix-pipe demo that proves the EDN+newline protocol
composes across N independent wat processes. Each stage reads one
typed shape from stdin, writes another typed shape to stdout. Same
discipline `:wat::kernel::spawn-program` exposes for in-process
spawning — the shell is the parent here.

| Script | Reads | Writes | Purpose |
|---|---|---|---|
| `router.wat` | `:demo::Event` | `:demo::Hit` | Drop events with `n <= 0`; forward positives. |
| `aggregator.wat` | `:demo::Hit` | `:demo::Partial` | Maintain running sum; emit after each. |
| `sink.wat` | `:demo::Partial` | `:demo::Total` | On EOF, emit the last partial as a final total. |

```bash
$ cat wat-scripts/events.edn \
    | cargo wat ./wat-scripts/router.wat \
    | cargo wat ./wat-scripts/aggregator.wat \
    | cargo wat ./wat-scripts/sink.wat

#demo/Total {:total 6}
```

The fixture (`events.edn`) is five `:demo::Event` lines; three are
hits (`n` = 1, 2, 3); the pipeline sums the hits and reports `6`.
Drop-cascade through the OS pipes mirrors the substrate's
crossbeam discipline: when the shell closes its end, each stage's
`read-line` returns `:None`, the program returns from its loop,
its stdout fd closes, and the next stage sees EOF.

## Adding a new script (current syntax)

Copy `intrinsic-metadata.wat` or a `fixes/*.wat` — those are live. The
current shape (the old `(stdin :IOReader)…` signature above is RETIRED):

1. Drop a new `.wat` file in this directory.
2. Define a nullary main: `(:wat::core::defn :user::main [] -> :wat::core::nil …)`.
   It takes no I/O params — I/O is ambient.
3. Read stdin with `(:wat::kernel::readln)` — no `-> :T` ascription (illegal
   on readln'; the decoded type flows from the consumer). Match
   `ReadlnOutcome::{Datum, Eof, Stopped}`. Read a file with
   `(:wat::io::read-file path)`.
4. Write with `(:wat::kernel::println v)` / `(:wat::kernel::eprintln v)` — both
   EDN-encode their argument. Open + stream + print; the substrate handles the rest.

The wat-cli binary already links every workspace `#[wat_dispatch]`
extension, so any path under `:wat::telemetry::*` /
`:wat::sqlite::*` / `:wat::lru::*` / `:wat::form::matches?` /
`:wat::time::*` works without a per-script Cargo.toml.

## Demo

```bash
cat wat-scripts/events.edn \
  | cargo wat ./wat-scripts/router.wat \
  | cargo wat ./wat-scripts/aggregator.wat \
  | cargo wat ./wat-scripts/sink.wat
#   → #demo/Total {:total 6}
```
