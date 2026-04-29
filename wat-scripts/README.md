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

| Script | Purpose |
|---|---|
| `seed-fixture.wat` | Write 5 sample Event::Log rows to the path on stdin (use this once to create a `.db` you can query with the other scripts). |
| `count-logs.wat` | Count Event::Log rows; print `logs: N`. |
| `metrics-summary.wat` | Count both Event::Log and Event::Metric rows in one script; print a one-line summary. Proves multiple streams can run sequentially off the same ReadHandle. |

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
