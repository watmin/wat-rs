# wat-rs arc 056 — `:wat::time::Instant` — INSCRIPTION

**Status:** shipped 2026-04-25. Half-day arc; landed in ~30 minutes
once the patterns from arcs 053 (`OnlineSubspace` /
`Reckoner` / `Engram` value variants) and arc 026 (math/stat
runtime-dispatch) were inhabited cleanly. ~140 LOC of Rust + 12
wat tests + USER-GUIDE rows.

Builder direction (from the lab's proofs lane):

> "we need time primitives. the requirement i have is having
> what you proposed and helpers who can do `(:time::from-iso8601
> x)` -> `:SomeTimeThing` and `(:time::to-iso8601 x N)` -> `:String`
> and having those iso8601 things support N fractional seconds"

> "Instant - better name"

> "is this :wat::std::time::* or just :wat::time::* - i view time
> as an IO so i think it should be next to :wat::io::*"

This arc is the second cross-lane handoff in the proofs/infra
split (the first was arc 027 — the RunDb shim). Lab proof 002
needs unique-per-execution timestamps for db filenames; this arc
ships them as a substrate primitive set. Once flipped through
the lab side, proof 002's `BLOCKED on wat-rs arc 056` status
becomes `ready`.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1     | `src/time.rs` (new) + `Cargo.toml` (chrono dep) + `runtime.rs` (Value::Instant + 9 dispatch arms) + `check.rs` (9 type schemes) | ~140 Rust + 1 dep | — | shipped |
| 2     | wat surface — N/A. Per BACKLOG slice 2's "thin re-export rather than a wrapper": runtime-dispatch primitives need no wrapper file. Same shape as `:wat::std::math::*` and `:wat::std::stat::*` — direct dispatch, no `wat/time.wat`. | 0 wat | — | omitted |
| 3     | `wat-tests/time.wat` (new) — 12 deftests | ~140 wat | 12 | shipped |
| 4     | This INSCRIPTION + USER-GUIDE row | doc-only | — | shipped |

**wat-rs wat_suite test count: 60 → 72. +12.**

Build: `cargo build --release` clean (single recompile after
chrono added). `cargo test --release wat_suite`: 72 tests, 0
failed, 0.27s.

---

## Architecture notes

### Single Value variant — `Value::Instant(DateTime<Utc>)`

Per arc 053 precedent for `Value::OnlineSubspace` /
`Value::Reckoner` / `Value::Engram`. The chrono `DateTime<Utc>`
is `Copy + Send + Sync + Clone + Debug` — no `ThreadOwnedCell`
wrapper needed (DateTime is just a 12-byte value; the
ThreadOwnedCell is for *mutable* substrate types that need
per-thread ownership for safe interior mutability under CSP).

`Value::type_name()` arm: `Value::Instant(_) => "wat::time::Instant"`.
Dispatch arms in `runtime.rs` delegate to nine `eval_time_*`
functions in the new `src/time.rs` module. Type schemes in
`check.rs` register the surface against `:wat::time::Instant`
as a path-typed opaque value.

### Q10 — `:wat::time::*`, not `:wat::std::time::*`

Per builder direction. `:wat::std::*` is the *pure* stdlib —
referentially-transparent algorithms (`sort-by`, `string::concat`,
`stat::mean`, `math::sqrt`). `:wat::io::*` is *world interaction*
— calls whose values depend on world state.

`(:wat::time::now)` observes the system clock — calling it twice
returns different values with no input. That's the canonical
signature of a side-effecting / world-observing primitive.
Belongs at the same nesting depth as `:wat::io::*`, not under
`:wat::std::*`.

The pure-time helpers (`at`, `at-millis`, `at-nanos`,
`from-iso8601`, `to-iso8601`, `epoch-*`) ARE referentially
transparent — they're pure converters — but they share a type
with the side-effecting `now`, and grouping reads more clearly
than splitting the time namespace across two prefixes.

### Q1 — Single type for "wall time" and "duration"

Per builder direction: collapse Rust's `SystemTime`/`Instant`
split into one Java/Clojure-style `Instant`. Duration measurement
is `(now)` before, `(now)` after, subtract integer accessors —
no separate `Duration` type. The wider language lineage (Java,
Clojure, JS, Python, SQL) treats time-of-day and elapsed-time as
the same shape; Rust's split is the outlier.

The cost: NTP can move the wall clock backwards mid-measurement.
For lab uses (~minutes-to-hours of computation; small
corrections), the simpler model wins. A `Stopwatch`-style
guaranteed-monotonic type can ship in a future arc when a caller
surfaces the need.

### Slice 2 omitted — runtime-dispatch primitives don't need a wat wrapper

The BACKLOG sketched `wat/time.wat` as a thin re-export and
called it optional. Looking at the existing `:wat::std::math::*`
and `:wat::std::stat::*` primitives, both ship purely as
runtime-dispatch arms with no wat surface file. Same shape adopted
for `:wat::time::*`. Saved 50 LOC of pure ceremony.

### `to-iso8601` formatting — hand-rolled fractional portion

chrono's `to_rfc3339_opts` supports `Secs / Millis / Micros /
Nanos / AutoSi` — fixed buckets, not arbitrary digit counts. Our
contract supports every digit count in `[0, 9]`, so the fractional
portion is hand-formatted (`format!("{}.{:0>width$}Z", ...)`)
after the integer datetime portion comes from `chrono`'s
`%Y-%m-%dT%H:%M:%S` formatter. `digits = 0` uses
`SecondsFormat::Secs` directly (chrono drops the fractional and
emits Z). Three lines of code; clean.

### `from-iso8601` returns `:Option<Instant>`

Per Q4 — same posture as `:wat::core::string::to-i64`. Parse
failure is a binary outcome from the caller's perspective; the
underlying chrono error message is rarely useful (callers reparse
or fall back). If a caller surfaces real need for the error
detail, a future arc widens to `:Result`.

### Construction-time errors

`at` / `at-millis` / `at-nanos` panic via `RuntimeError::TypeMismatch`
when the input is out of chrono's representable range
(unrepresentably-far-future / pre-epoch — chrono spans roughly
year -262144 to +262144, so this rarely triggers). `epoch-nanos`
panics when the instant falls outside the i64-nanosecond range
(~1677 to ~2262). All documented per `feedback_shim_panic_vs_option`.

---

## What this unblocks

- **Lab proof 002 — thinker baseline.** Generates timestamped DB
  filenames so re-runs accumulate rather than PK-violate. Status
  flips from `BLOCKED on wat-rs arc 056` to `ready` in the lab
  side's proof doc.
- **Future lab proofs / benchmarks / log emitters.** Any wat
  program needing unique-per-execution discriminators or
  wall-clock log records.
- **Future cache-eviction / TTL policies.** Wall-clock now()
  enables time-based expiration logic.
- **Cross-process / cross-machine handoff.** ISO 8601 round-trip
  enables serializing Instants across boundaries (JSON, SQL
  TIMESTAMP, file metadata).
- **Future scheduler / timer primitives.** Instant is the natural
  consumer type for any future `(:wat::kernel::sleep-until ...)`.

---

## What this arc deliberately did NOT add

Reproduced from DESIGN's "What this arc does NOT add":

- **A monotonic / `Stopwatch` type.** Single-type model per Q1.
- **A `Duration` value type.** Duration is `i64` ns/ms/s by
  convention.
- **Time zones beyond UTC.** Future arc.
- **Locale-aware formatting / non-ISO formats.** ISO 8601 only.
- **Calendar arithmetic** (add a month, week-of-year, etc.).
  Future arc.
- **`sleep` / `delay` primitives.** Out of scope; lives near
  kernel/scheduler work.

---

## The thread

- **2026-04-25** — DESIGN.md + BACKLOG.md drafted by the lab's
  proofs lane in `docs/arc/2026/04/056-time-instant/`.
- **2026-04-25** (this session) — slices 1, 3, 4 + INSCRIPTION
  shipped same day. Slice 2 (wat surface) deliberately omitted —
  runtime-dispatch primitives need no wrapper file.
- Lab side next: flip proof 002 from `BLOCKED on wat-rs arc 056`
  to `ready` and write the supporting program with timestamped
  DB filenames.

PERSEVERARE.
