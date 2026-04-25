# wat-rs arc 056 — `:wat::time::Instant` — BACKLOG

**Shape:** four slices. Rust runtime + Cargo dep first; wat surface
second; tests third; INSCRIPTION + USER-GUIDE row fourth. Total
estimate: half a day.

This arc unblocks holon-lab-trading proof 002 (timestamped DB
filenames). Once shipped, the lab side flips proof 002's status
from BLOCKED to ready and writes the supporting program.

---

## Slice 1 — Rust runtime + Cargo dep

**Status: shipped 2026-04-25.**

`wat-rs/Cargo.toml` — append to `[dependencies]`:

```toml
chrono = "0.4"
```

`wat-rs/src/time.rs` (new) — the implementations:

```rust
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

// Either a new Value variant (per arc 053 precedent for
// OnlineSubspace / Reckoner) or a #[wat_dispatch] newtype.
// Implementer's choice; pick the lighter path.

pub fn now() -> /* Instant value */ { ... }

pub fn at(epoch_seconds: i64) -> /* Instant value */ {
    Utc.timestamp_opt(epoch_seconds, 0).single()
        .expect("at: epoch_seconds out of representable range")
}

pub fn at_millis(epoch_ms: i64) -> /* Instant value */ { ... }

pub fn at_nanos(epoch_ns: i64) -> /* Instant value */ { ... }

pub fn from_iso8601(s: &str) -> Option</* Instant value */> {
    DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc))
}

pub fn to_iso8601(i: &DateTime<Utc>, digits: i64) -> String {
    let digits = digits.clamp(0, 9) as u32;
    let secs_part = i.format("%Y-%m-%dT%H:%M:%S");
    if digits == 0 {
        format!("{}Z", secs_part)
    } else {
        let nanos = i.timestamp_subsec_nanos();
        let scaled = nanos / 10_u32.pow(9 - digits);
        format!("{}.{:0>width$}Z", secs_part, scaled, width = digits as usize)
    }
}

pub fn epoch_seconds(i: &DateTime<Utc>) -> i64 { i.timestamp() }
pub fn epoch_millis(i: &DateTime<Utc>)  -> i64 { i.timestamp_millis() }
pub fn epoch_nanos(i: &DateTime<Utc>)   -> i64 {
    i.timestamp_nanos_opt()
        .expect("epoch_nanos: instant out of i64-nanosecond range (year ~2262)")
}
```

`wat-rs/src/runtime.rs` — add dispatch arms:

```rust
":wat::time::now"            => eval_time_now(...),
":wat::time::at"             => eval_time_at(...),
":wat::time::at-millis"      => eval_time_at_millis(...),
":wat::time::at-nanos"       => eval_time_at_nanos(...),
":wat::time::from-iso8601"   => eval_time_from_iso8601(...),
":wat::time::to-iso8601"     => eval_time_to_iso8601(...),
":wat::time::epoch-seconds"  => eval_time_epoch_seconds(...),
":wat::time::epoch-millis"   => eval_time_epoch_millis(...),
":wat::time::epoch-nanos"    => eval_time_epoch_nanos(...),
```

`wat-rs/src/check.rs` — register schemes for each:

```rust
env.register(":wat::time::now",
  TypeScheme { params: vec![], ret: instant_ty(), ... });

env.register(":wat::time::at",
  TypeScheme { params: vec![i64_ty()], ret: instant_ty(), ... });

// ... and so on for the remaining seven.
```

Type registration: a new built-in type `wat::time::Instant`,
analogous to how Phase 4 substrate types (Reckoner,
OnlineSubspace) register.

**Estimated cost:** ~120 LOC + Cargo.toml line. ~2.5 hours.

---

## Slice 2 — Wat surface

**Status: omitted 2026-04-25.** Runtime-dispatch primitives don't need a wat wrapper — same shape as `:wat::std::math::*` and `:wat::std::stat::*` (no `wat/std/math.wat` or `wat/std/stat.wat` files exist; both ship as direct dispatch arms). Saved ~50 LOC of pure ceremony. INSCRIPTION captures the rationale.

`wat-rs/wat/time.wat` (new) — sits at the same nesting depth as
`wat/io/`, `wat/std/`, etc. Per Q10 (namespace-as-IO), time is a
top-level concern rather than nested under std.

```scheme
;; wat/time.wat — :wat::time::Instant + constructors / accessors / formatter.
;;
;; Wall-clock time as a single value type. Java/Clojure lineage:
;; one Instant covers both "when did this happen?" and "how long
;; did this take?" The latter is `(now)` before, `(now)` after,
;; subtract the integer accessors.
;;
;; UTC only. ISO 8601 / RFC 3339 round-trips. Sub-second precision
;; up to nanoseconds. i64 nanos saturates at year ~2262.
;;
;; Time is a world-observing primitive (calls to `now` differ
;; based on the system clock); namespace placement reflects that —
;; `:wat::time::*` sits next to `:wat::io::*`, not under
;; `:wat::std::*` (which connotes referentially-transparent
;; stdlib utilities).

(:wat::core::define
  (:wat::time::now -> :wat::time::Instant)
  ...)

(:wat::core::define
  (:wat::time::at (epoch-seconds :i64) -> :wat::time::Instant)
  ...)

;; ... at-millis, at-nanos, from-iso8601, to-iso8601, epoch-*.
```

(If the slice-1 implementation surfaces these as direct
runtime-dispatch primitives — same as `:wat::core::+` — the
`wat/time.wat` file may be a thin re-export rather than a
wrapper. Either is fine; pick during implementation.)

**Estimated cost:** ~50 LOC. ~30 minutes.

---

## Slice 3 — Tests

**Status: shipped 2026-04-25.**

`wat-rs/wat-tests/time.wat` (new) — 8-10 deftests covering each
primitive.

Test budget:

1. `test-now-returns-instant` — `(now)` returns a value typed
   `:wat::time::Instant`; epoch-seconds is in [2020-01-01,
   2100-01-01] range.
2. `test-at-zero-is-epoch` — `(at 0)` formatted at digits=0 is
   `"1970-01-01T00:00:00Z"`.
3. `test-at-millis-vs-at` — `(at-millis 1000) == (at 1)` via
   epoch-seconds equality.
4. `test-at-nanos-vs-at-millis` — `(at-nanos 1_000_000_000) ==
   (at-millis 1000)`.
5. `test-iso8601-roundtrip-3-digits` — parse
   `"2026-04-25T14:30:42.123Z"` and format at digits=3 yields the
   identical string.
6. `test-iso8601-roundtrip-9-digits` — same with nanosecond
   precision.
7. `test-iso8601-parse-failure` — `(from-iso8601 "garbage")`
   returns `:None`.
8. `test-to-iso8601-digits-0` — `(at 0) → "1970-01-01T00:00:00Z"`
   (no fractional part).
9. `test-to-iso8601-digits-clamp-high` — digits=42 clamps to 9.
10. `test-to-iso8601-digits-clamp-low` — digits=-5 clamps to 0.
11. `test-elapsed-via-subtract` — `(now)` twice; second's
    epoch-millis ≥ first's epoch-millis.
12. `test-epoch-accessors-consistency` — `epoch-seconds * 1000 +
    sub-second-millis-portion == epoch-millis` for an arbitrary
    Instant.

**Estimated cost:** ~80 LOC + 12 tests. ~1.5 hours.

---

## Slice 4 — INSCRIPTION + USER-GUIDE row

**Status: shipped 2026-04-25.** USER-GUIDE row added under `:wat::time::*`. FOUNDATION-CHANGELOG note skipped — the file doesn't exist in this repo.

- **INSCRIPTION.md** — record what landed (LOC delta, test
  count delta), confirm Q10's namespace decision in practice,
  note any chrono-version surprises.
- **USER-GUIDE.md** — add a row in the "Forms appendix" table
  documenting the `:wat::time::*` surface. Same shape as
  existing entries.
- **FOUNDATION-CHANGELOG row** — single line documenting time
  primitives now ship under `:wat::time::*` (not std::), with
  the world-interaction rationale.

Notify holon-lab-trading via the proof-002 stub:
`docs/proofs/2026/04/002-thinker-baseline/PROOF.md` flips its
BLOCKED status when the lab picks up this arc's outputs.

**Estimated cost:** ~1 hour. Doc only.

---

## Verification end-to-end

After all slices land:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --test test 2>&1 | grep "::time::"
```

Should show 12 deftests passing. Total wat-rs test count
~970 → ~982.

```scheme
;; And in any wat program with the substrate:
(:wat::core::let*
  (((i :wat::time::Instant) (:wat::time::now))
   ((s :String) (:wat::time::to-iso8601 i 3)))
  (:wat::io::IOWriter/println stdout s))
;; Prints something like "2026-04-25T14:30:42.123Z\n".
```

---

## Out of scope

- **A separate Monotonic / `Stopwatch` type.** Per Q1, single-
  type model. Future arc when a caller surfaces a need for
  guaranteed-monotonic intervals.
- **A `Duration` value type.** Duration is integer ns/ms/s; no
  separate type.
- **Time zones beyond UTC.** Future arc.
- **Calendar arithmetic** (add a month, week-of-year, etc.).
  Future arc.
- **`sleep` / `delay` primitives.** Out of scope; lives near
  kernel/scheduler work.
- **Locale-aware formatting / non-ISO formats.** ISO 8601 only.

---

## Risks

**chrono version coupling.** `chrono = "0.4"` is the current
stable major; major-version bumps are rare but real. If chrono
0.5 lands during this arc's life, pin to 0.4 explicitly and
defer the upgrade.

**`epoch-nanos` overflow at year 2262.** Documented; for any
caller operating outside that range, panics on construction.
Future arc could widen to i128 once wat has i128 support (not
a 2026-priority concern).

**Time-zone surprises in `from-iso8601`.** chrono's
`parse_from_rfc3339` accepts strings with explicit offsets like
`+05:30`; we convert all to UTC for storage. A caller expecting
to recover the original offset from a parsed Instant is wrong —
the offset is consumed during parse. If that's needed, future
arc adds a `:wat::time::ZonedInstant` or similar. v1 doesn't
support it.

---

## Total estimate

- Slice 1: 2.5 hours (Rust runtime + Cargo dep)
- Slice 2: 30 minutes (wat surface)
- Slice 3: 1.5 hours (tests)
- Slice 4: 1 hour (INSCRIPTION + docs)

**~5.5 hours = a half day of focused work.** Lighter than arc
055 (recursive patterns); same shape as arc 050 (polymorphic
arithmetic) and arc 046 (numeric primitives).

---

## What this unblocks

- **Lab proof 002** — flips from BLOCKED to ready; supporting
  program writes timestamped DB filenames so re-runs accumulate
  rather than PK-violate.
- **Future lab proofs / benchmarks** — any wat consumer needing
  unique-per-execution discriminators or wall-clock log
  records.
- **Lab run-ledger work** — per-run timestamps in `runs/`
  metadata become trivially derivable.
- **Cross-process / cross-machine handoff** — ISO 8601 round-
  trip enables serializing Instants across process boundaries
  (JSON, SQL TIMESTAMP, file metadata).
- **Future cache eviction / TTL policies** — wall-clock now()
  enables time-based expiration.
- **Future scheduler / timer primitives** — Instant is the type
  any future `(:wat::kernel::sleep-until ...)` would consume.

PERSEVERARE.
