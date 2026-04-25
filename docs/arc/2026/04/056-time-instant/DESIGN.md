# wat-rs arc 056 — `:wat::time::Instant`

**Status:** opened 2026-04-25.

**Scope:** small. ~120 LOC Rust + ~50 LOC wat surface + tests.
Same shape as the math/stat carry-alongs from arcs 025/026.
Estimate ~half a day.

Builder direction (from a coordination conversation in the lab's
proofs lane):

> "we need time primitives. the requirement i have is having
> what you proposed and helpers who can do ... `(:time::from-iso8601
> x)` -> `:SomeTimeThing` and `(:time::to-iso8601 x N)` -> `:String`
> and having those iso8601 things support N fractional seconds
> (i typically prefer 3 digits - support N) ... we also need some
> funcs who do something like `(:time::now)` -> `:SomeTimeThing`
> `(:time::at N)` -> `:SomeTimeThing` where at is epoch seconds"

> "Instant - better name"

> "confirming instant as the clojure side -- i legit have no idea
> what walltime even means - i don't care... if we need to measure
> duration we'll measure 'now at time point in time' against 'now
> at some future time'"

The lineage is **Clojure / Java**: `java.time.Instant` is a point
in time on the wall clock. Single type. Duration measurement is
two `now` calls subtracted. Rust's split (`SystemTime` vs
`Instant`) is the outlier; this arc doesn't import it.

---

## Why this arc, why now

Lab proof 002 (and every future proof / log emitter / run-id
discriminator) needs unique-per-execution timestamps for db
filenames, log records, ISO 8601 strings. wat has no time
primitives; today, the only path is to plumb a Rust-side
timestamp through env vars or a per-shim extension. Both leak
language details into call sites.

The clean answer: a substrate primitive set under
`:wat::time::*`. Once shipped, every wat consumer (proofs,
benchmarks, log emitters, future cache eviction policies, future
run ledgers) uses the same surface. Same carry-along pattern as
`sort-by` (arc 025 slice 1), `not=` + Enum equality (arc 025
slice 2), `sqrt` + `std::stat::*` (arc 026).

Cross-references:
- [`docs/proofs/2026/04/002-thinker-baseline/PROOF.md`](https://github.com/watmin/holon-lab-trading/blob/main/docs/proofs/2026/04/002-thinker-baseline/PROOF.md) — the immediate consumer in the lab.
- Rust `chrono::DateTime<Utc>` — the implementation backing.
- Java `java.time.Instant` — the naming lineage; semantically equivalent.
- arc 053 (Phase 4 substrate) — precedent for `Value::*` variants over external types via `#[wat_dispatch]`.

---

## What ships

A single new value type and a small surface around it.

### The type

```scheme
;; Opaque value type. Internals: Rust chrono::DateTime<Utc>.
;; Wat code never inspects fields directly — uses the
;; constructors/accessors below.
:wat::time::Instant
```

Implementation: native `Value::Instant(DateTime<Utc>)` variant in
wat-rs's runtime, mirroring how arc 053 surfaced
`Value::OnlineSubspace` and `Value::Reckoner`. Either that or a
`#[wat_dispatch]` newtype — pick the lighter path during
implementation.

### Constructors (5)

```scheme
(:wat::time::now              -> :Instant)
;; Current wall-clock time.

(:wat::time::at             (epoch-seconds :i64) -> :Instant)
;; From integer seconds since 1970-01-01T00:00:00Z.

(:wat::time::at-millis      (epoch-ms :i64) -> :Instant)
;; From integer milliseconds since epoch.

(:wat::time::at-nanos       (epoch-ns :i64) -> :Instant)
;; From integer nanoseconds since epoch. (i64 nanos overflow at
;; ~2262; documented; use at/at-millis for far-future construction.)

(:wat::time::from-iso8601   (s :String) -> :Option<Instant>)
;; Parse an ISO 8601 string. Returns :None on parse failure.
;; Accepts the chrono::DateTime<Utc>::parse_from_rfc3339 grammar
;; (RFC 3339 is the practical ISO 8601 subset everyone uses).
```

### Formatter (1)

```scheme
(:wat::time::to-iso8601 (i :Instant) (digits :i64) -> :String)
;; ISO 8601 / RFC 3339 with N fractional second digits.
;;   digits =  0  →  "2026-04-25T14:30:42Z"
;;   digits =  3  →  "2026-04-25T14:30:42.123Z"     ← preferred default
;;   digits =  6  →  "2026-04-25T14:30:42.123456Z"
;;   digits =  9  →  "2026-04-25T14:30:42.123456789Z"
;; Out-of-range digits clamp to [0, 9]. Output always UTC; the
;; trailing 'Z' encodes that.
```

### Accessors (3)

```scheme
(:wat::time::epoch-seconds (i :Instant) -> :i64)
(:wat::time::epoch-millis  (i :Instant) -> :i64)
(:wat::time::epoch-nanos   (i :Instant) -> :i64)
;; Truncating, not rounding. Sub-second precision lost in
;; epoch-seconds; sub-millisecond lost in epoch-millis; etc.
```

### Duration measurement — by composition, not a new verb

The Clojure/Java idiom: `(now)` before, `(now)` after, subtract
the integer accessors. No `elapsed-*` helper needed.

```scheme
((start :Instant) (:wat::time::now))
;; ... do work ...
((end :Instant)   (:wat::time::now))
((elapsed-ms :i64)
 (:wat::core::- (:wat::time::epoch-millis end)
                (:wat::time::epoch-millis start)))
```

`wat::core::-` already exists for i64. No new combinator.

---

## Decisions resolved

### Q1 — Why not split into wall-clock + monotonic types?

**Per builder direction — collapse to one type.** Rust's
`SystemTime` vs `Instant` split is unusual; the broader lineage
(Java `java.time.Instant`, Clojure's
`(System/currentTimeMillis)`, JS `Date`, Python `datetime`,
SQL `TIMESTAMP`) is one type for "a point in time." Wat follows
the broader lineage.

The cost: NTP can move the clock backwards. Subtracting two
Instants might yield a negative or wrong duration if NTP
corrects mid-measurement. For the lab's uses (~minutes-to-hours
of computation; NTP corrections rare and small), the simpler
single-type model wins. If a future caller surfaces "I need
guaranteed-monotonic intervals," that's its own arc.

### Q2 — Backing type: `chrono::DateTime<Utc>` or `std::time::SystemTime`?

**`chrono::DateTime<Utc>`.** Reasons:

- `std::time::SystemTime` has no `Display` impl that produces ISO
  8601; we'd hand-format. chrono does it.
- chrono's RFC 3339 parser handles every reasonable input format
  (with/without fractional seconds, +HH:MM offsets, Z suffix).
  std-only would require pulling in `time` crate or hand-rolling.
- chrono's `format!("%.3f")` style isn't quite what we want for
  "exactly N digits" — we'll hand-format the fractional portion,
  but the integer-part formatting (year/month/day/hour/min/sec)
  comes free.
- chrono is already in wat-rs's transitive dep tree via the
  lab's parquet shim. No new top-level dep.

### Q3 — Cargo.toml — direct dep on chrono?

**Yes.** wat-rs gains `chrono = "0.4"` in its own `Cargo.toml`
(direct dep, not transitive). Standard for any Rust project that
needs human-readable wall-clock time. ~150 KB compiled in
release — negligible.

### Q4 — `from-iso8601` return type: `Option` or `Result`?

**`Option<Instant>`.** Parse failure is a binary "valid /
invalid" outcome from the caller's perspective; the underlying
chrono error carries information about *what* was malformed but
the caller almost never wants it (you reparse with a different
input or fall back). Same posture as `:wat::core::string::to-i64`
returning `:Option<i64>`. If a caller surfaces a real need for
the error message, future arc widens to `Result`.

### Q5 — Out-of-range fractional digits

**Clamp silently to `[0, 9]`.** Negative or `>9` digits get
clamped at the boundary. No panic. (Rationale: the call site
that produced `digits = 12` is buggy, but a panic in time
formatting is the wrong place to enforce — the result still
makes sense at 9 digits.)

### Q6 — Time zones / non-UTC

**UTC only for v1.** All Instants are UTC; ISO 8601 output
ends with `Z`; `from-iso8601` accepts any offset and converts
to UTC for storage. No `LocalDateTime` / time-zone-aware type.
Future arc adds zoned variants when a caller surfaces them.

### Q7 — Daylight saving / leap seconds

**Out of scope.** chrono::DateTime<Utc> handles UTC monotonically
within itself (no DST). Leap seconds are not represented (chrono
ignores them — same as POSIX time). For the lab's uses (filename
discriminators, paper-resolution timestamps, run-id allocation),
not a concern.

### Q8 — Integer overflow on epoch-nanos

**Document; don't guard.** i64 nanoseconds since 1970 saturates
~year 2262. The cap is documented in `at-nanos` and `epoch-nanos`
docstrings. Construction or accessor at out-of-range values
panics with a clear message rather than silently truncating.
For most callers operating in the ~2020s-2100s range, no concern.

### Q9 — Naming the type — `Instant` vs `WallTime` vs `Time`?

**`Instant`.** Per builder direction. Java's
`java.time.Instant` precedent. Clojure / JVM ecosystem.
Documented in module-level header comment that this is
*wall-clock* (despite Rust's `std::time::Instant` being
monotonic) — the lineage choice is explicit.

### Q10 — Namespace placement: `:wat::std::time::*` vs `:wat::time::*`?

**`:wat::time::*` — sibling of `:wat::io::*`, not nested under
`:wat::std::*`.**

Per builder direction:

> "is this :wat::std::time::* or just :wat::time::* - i view time
> as an IO so i think it should be next to :wat::io::*"

The reasoning is real. `:wat::std::*` is the **pure stdlib**:
algorithms, data structures, math primitives. Calls under
`:wat::std::*` are referentially transparent — same input, same
output, no observable side effects. `sort-by`, `string::concat`,
`stat::mean`, `math::sqrt`, `list::map` all qualify.

`:wat::io::*` is **world interaction** — reading from / writing
to streams that exist outside the program (stdin, stdout,
stderr, files via Loader). Calls under `:wat::io::*` are
side-effecting; their values depend on the world's state.

`(:wat::time::now)` is firmly in the second category. It
observes the system clock — a piece of world state that changes
between calls. Calling `now` twice can return different values
even with no input. That's the canonical signature of a side-
effecting / world-observing primitive.

So `:wat::time::*` belongs alongside `:wat::io::*` at the same
nesting depth, not under `:wat::std::*` which connotes
referential transparency.

The pure-time helpers (`at`, `at-millis`, `at-nanos`,
`from-iso8601`, `to-iso8601`, `epoch-*`) are in fact
referentially transparent — they're just pure converters. But
they share a type with the side-effecting `now`, and grouping
them with `now` reads more clearly than splitting "the time
namespace" across two top-level prefixes.

---

## Implementation sketch

Three files.

### `wat-rs/src/time.rs` (new) — Rust impls

```rust
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};

// New Value variant (or #[wat_dispatch] newtype) holding DateTime<Utc>.

fn now() -> Value::Instant { ... }
fn at(epoch_secs: i64) -> Value::Instant { ... }
fn at_millis(epoch_ms: i64) -> Value::Instant { ... }
fn at_nanos(epoch_ns: i64) -> Value::Instant { ... }
fn from_iso8601(s: String) -> Value::Option<Instant> { ... }

fn to_iso8601(i: &Instant, digits: i64) -> String {
    let digits = digits.clamp(0, 9) as usize;
    // Hand-format: integer datetime + . + N digits + Z
    let secs_part = i.format("%Y-%m-%dT%H:%M:%S");
    if digits == 0 {
        format!("{}Z", secs_part)
    } else {
        let nanos = i.timestamp_subsec_nanos();
        let scaled = nanos / 10_u32.pow(9 - digits as u32);
        format!("{}.{:0>width$}Z", secs_part, scaled, width = digits)
    }
}

fn epoch_seconds(i: &Instant) -> i64 { i.timestamp() }
fn epoch_millis(i: &Instant)  -> i64 { i.timestamp_millis() }
fn epoch_nanos(i: &Instant)   -> i64 { i.timestamp_nanos_opt().expect("year out of range") }
```

### `wat-rs/wat/time.wat` (new) — wat surface

Thin wrappers; same shape as `wat/std/test.wat`'s `assert-eq`-
class wrappers around runtime primitives.

### `wat-rs/wat-tests/time.wat` (new) — tests

8-10 deftests covering each primitive and each interesting edge:
- `now` returns an Instant.
- `at 0` is the epoch (formatter outputs `1970-01-01T00:00:00Z`).
- `at-millis 1000 == at 1`.
- `from-iso8601` round-trip on a valid string.
- `from-iso8601` returns None on garbage.
- `to-iso8601` with digits=0/3/6/9 produces the expected formats.
- `to-iso8601` clamps digits=-1 to 0 and digits=42 to 9.
- Duration measurement via two `now` + accessor subtract.

---

## Tests — minimum

```scheme
(:wat::test::deftest :wat-rs::test::time::now-returns-instant
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::now))
     ((s :i64) (:wat::time::epoch-seconds i)))
    ;; Sanity: now is in the post-2020-pre-2100 range.
    (:wat::test::assert-eq (:wat::core::> s 1577836800) true)))    ; 2020-01-01

(:wat::test::deftest :wat-rs::test::time::at-zero-is-epoch
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ((s :String) (:wat::time::to-iso8601 i 0)))
    (:wat::test::assert-eq s "1970-01-01T00:00:00Z")))

(:wat::test::deftest :wat-rs::test::time::iso8601-roundtrip
  ()
  (:wat::core::let*
    (((parsed :Option<wat::time::Instant>)
      (:wat::time::from-iso8601 "2026-04-25T14:30:42.123Z")))
    (:wat::core::match parsed -> :()
      ((Some i)
        (:wat::core::let*
          (((s :String) (:wat::time::to-iso8601 i 3)))
          (:wat::test::assert-eq s "2026-04-25T14:30:42.123Z")))
      (:None
        (:wat::kernel::assertion-failed!
          "from-iso8601 returned None for valid input" :None :None)))))

(:wat::test::deftest :wat-rs::test::time::digits-clamp
  ()
  (:wat::core::let*
    (((i :wat::time::Instant) (:wat::time::at 0))
     ((s :String) (:wat::time::to-iso8601 i 42)))    ; clamps to 9
    (:wat::test::assert-eq s "1970-01-01T00:00:00.000000000Z")))

;; Plus 4-6 more for at-millis/at-nanos round-trips, epoch-* accessor
;; consistency, parse-failure case, and elapsed-via-subtraction.
```

---

## What this arc does NOT add

- **A monotonic / `Instant`-equivalent-of-Rust type.** Defer
  per Q1 / builder direction.
- **A `Duration` value type.** Duration is i64 ns/ms/s by
  convention; no separate type. Future arc if a caller wants
  arithmetic-safe duration math.
- **Time zones.** UTC only.
- **Locale-aware formatting.** ISO 8601 only; no day-of-week
  names, no localized strings.
- **Calendar arithmetic** (add a month, etc.). Future arc;
  chrono provides this and the surface can grow.
- **Sleep / delay primitives.** Out of scope; lives elsewhere
  (likely future kernel / scheduler arc).

---

## Non-goals

- **Sub-nanosecond precision.** chrono caps at ns; that's our
  cap.
- **Pre-1970 instants.** `at` accepts negative seconds (chrono
  handles them); behavior beyond that is "whatever chrono does."
  Documented.
- **Performance optimization.** Time primitives are not hot
  paths in any current consumer.

---

## What this unblocks

- **Lab proof 002 — thinker baseline.** Generates timestamped DB
  filenames so re-runs accumulate rather than PK-violate.
- **Future proofs / benchmarks.** Any wat program that needs
  unique-per-execution discriminators or wall-clock log
  records.
- **Future cache eviction policies.** TTL-based caches need
  wall-clock now().
- **Future run-ledger work.** Per-run timestamp metadata in the
  lab's runs/ tree.
- **Cross-machine handoff.** ISO 8601 round-trip enables
  serializing/deserializing Instants across processes (e.g.,
  to/from JSON, SQL TIMESTAMP columns, file metadata).

PERSEVERARE.
