# `wat-scripts/` — interrogation scripts for telemetry `.db` files

Pry/gdb-style ad-hoc scripts that operate on the sqlite-backed
telemetry `.db` files arc 091's writer + arc 093's reader path
ship. Each script is a standalone wat program with a
`:user::main` that:

1. Reads a `.db` path from stdin (one line via
   `:wat::io::IOReader/read-line`).
2. Opens the path with `:wat::sqlite::open-readonly`.
3. Runs queries via `:wat::telemetry::sqlite/stream-{logs,metrics}`
   + `:wat::std::stream::*` combinators.
4. Prints results to stdout.

The scripts run against the bundled batteries-included `wat`
binary (`crates/wat-cli/`, arc 099) — no per-script build step
needed. A consumer treats wat-rs like ruby for the duration of
this session.

## Running

Direct shell-pipe:

```bash
echo /path/to/run.db | ./target/release/wat ./wat-scripts/count-logs.wat
```

Or via the convenience wrapper at `scripts/query-db.sh` (handles
abs-path resolution + auto-builds the binary on first invocation):

```bash
./scripts/query-db.sh /path/to/run.db ./wat-scripts/count-logs.wat
```

## Scripts

### Telemetry interrogation (arc 093)

| Script | Purpose |
|---|---|
| `seed-fixture.wat` | Write 5 sample Event::Log rows to the path on stdin (use this once to create a `.db` you can query with the other scripts). |
| `count-logs.wat` | Count Event::Log rows; print `logs: N`. |
| `metrics-summary.wat` | Count both Event::Log and Event::Metric rows in one script; print a one-line summary. Proves multiple streams can run sequentially off the same ReadHandle. |

### Bidirectional ping-pong (arc 103a in operational form)

A wat program spawns a wat program. They exchange `:demo::Ping` /
`:demo::Pong` messages over kernel pipes for N round trips; both
shut down cleanly when the conversation ends. The mini-TCP
discipline from `docs/ZERO-MUTEX.md` §"Mini-TCP via paired
channels" — same shape, transported over `pipe(2)` instead of
crossbeam channels.

| Script | Purpose |
|---|---|
| `ping-pong.wat` | Parent. Spawns `pong.wat`, sends Ping, reads Pong, repeats 5 times, closes child stdin, joins. |
| `pong.wat` | Child responder. Reads each Ping, mirrors the n in a Pong, recurses. EOF → exit. |

```bash
$ ./target/release/wat ./wat-scripts/ping-pong.wat
round 1: ping → pong
round 2: ping → pong
round 3: ping → pong
round 4: ping → pong
round 5: ping → pong
done — 5 round trips
```

The shape:

```
wat-cli (Rust binary)
  └─ ping-pong.wat (frozen world A)
       ├─ stdin/stdout/stderr → real OS handles
       └─ :wat::kernel::spawn-program (./pong.wat)
            └─ pong.wat (frozen world B, on a thread)
                 └─ stdin/stdout/stderr → 3 OS pipe ends
```

Two wat programs, two frozen worlds, three OS pipes. Neither side
can reach into the other's bindings; both share the binary's Rust
shims; communication only crosses the pipe surface. Bidirectional
back-pressure paces every round-trip.

### EDN-stdin dispatcher (arc 103c)

The hologram-nesting pattern from arc 103a, made operational. The
dispatcher reads one `#demo/Job` EDN line from stdin, reads the
named query-program's source via `:wat::io::read-file`, spawns it
via `:wat::kernel::spawn-program` with the db-path piped in as the
inner's stdin, forwards the inner's stdout to the dispatcher's own
stdout. Two wat programs, two frozen worlds, three OS pipes
between them.

| Script | Purpose |
|---|---|
| `dispatch.wat` | Read `#demo/Job {:db-path :query-program}`, spawn the named program, mediate IO. |

```bash
$ echo /tmp/dispatch-demo.db | ./target/release/wat ./wat-scripts/seed-fixture.wat
seeded 5 logs to: /tmp/dispatch-demo.db

$ echo '#demo/Job {:db-path "/tmp/dispatch-demo.db" :query-program "./wat-scripts/count-logs.wat"}' \
    | ./target/release/wat ./wat-scripts/dispatch.wat
logs: 5

$ echo '#demo/Job {:db-path "/tmp/dispatch-demo.db" :query-program "./wat-scripts/metrics-summary.wat"}' \
    | ./target/release/wat ./wat-scripts/dispatch.wat
logs: 5  metrics: 0
```

The inner programs (`count-logs.wat`, `metrics-summary.wat`) run
in their own frozen worlds — they cannot see the dispatcher's
bindings or symbol table. They share the binary's Rust shims
(`:wat::sqlite::*`, etc.) but otherwise communicate only through
the three OS pipes the dispatcher allocated.

See `docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md` for the
framing — this is the hologram model in operational form.

### Pipeline composition (arc 103a)

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
    | ./target/release/wat ./wat-scripts/router.wat \
    | ./target/release/wat ./wat-scripts/aggregator.wat \
    | ./target/release/wat ./wat-scripts/sink.wat

#demo/Total {:total 6}
```

The fixture (`events.edn`) is five `:demo::Event` lines; three are
hits (`n` = 1, 2, 3); the pipeline sums the hits and reports `6`.
Drop-cascade through the OS pipes mirrors the substrate's
crossbeam discipline: when the shell closes its end, each stage's
`read-line` returns `:None`, the program returns from its loop,
its stdout fd closes, and the next stage sees EOF.

## Adding a new script

1. Drop a new `.wat` file in this directory.
2. Define `:user::main` with the standard `(stdin :wat::io::IOReader)
   (stdout :wat::io::IOWriter) (stderr :wat::io::IOWriter) -> :()`
   signature.
3. `(:wat::io::IOReader/read-line stdin)` for the path; pattern-match
   the `:Option<String>` for `:None` (no input given) vs `(Some
   path)`.
4. Open + stream + print. The substrate handles the rest.

The wat-cli binary already links every workspace `#[wat_dispatch]`
extension, so any path under `:wat::telemetry::*` /
`:wat::sqlite::*` / `:wat::lru::*` / `:wat::form::matches?` /
`:wat::time::*` works without a per-script Cargo.toml.

## Demo

```bash
# 1) Seed a fixture .db.
echo /tmp/demo.db | ./target/release/wat ./wat-scripts/seed-fixture.wat
#   → seeded 5 logs to: /tmp/demo.db

# 2) Query it.
./scripts/query-db.sh /tmp/demo.db ./wat-scripts/count-logs.wat
#   → logs: 5

./scripts/query-db.sh /tmp/demo.db ./wat-scripts/metrics-summary.wat
#   → logs: 5  metrics: 0
```
