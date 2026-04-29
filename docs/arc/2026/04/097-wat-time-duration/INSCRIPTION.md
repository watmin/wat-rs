# Arc 097 — `:wat::time::Duration` + arithmetic + ActiveSupport
helpers — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate gained a Duration runtime variant, polymorphic
Instant ± Duration arithmetic, and ActiveSupport-flavored
`ago` / `from-now` composers (with 14 pre-composed unit sugars).
Across four slices, time math in wat scripts moved from raw nanos
arithmetic to:

```scheme
(:wat::time::- some-instant (:wat::time::Hour 1))
(:wat::time::- later-instant earlier-instant)   ; -> Duration
(:wat::time::hours-ago 1)
(:wat::time::days-from-now 2)
```

Same shape Ruby's ActiveSupport ships. Sibling arc to
[arc 093](../093-wat-telemetry-workquery/DESIGN.md) (telemetry
interrogation), which depends on these for `Since` / `Until`
constraint ergonomics.

**Predecessors:**
- Arc 056 — `:wat::time::Instant` (single value type for wall-
  clock points; original "no separate Duration" decision that this
  arc reverses).
- Arc 048 — user-defined enums (Value runtime variants pattern).
- Arc 050 — polymorphic arithmetic dispatch precedent.

**Surfaced by:** user direction 2026-04-29 mid-arc-093 design:

> "and we'll never use `(:wat::telemetry::since hour-ago)` — we'll
> use iso8601 timestamps to nano ints — right? ... Ruby Active's
> helpers for time are a wonderful ux..."

> "I figured it'd be time -> forms -> clara"

> "uh... let's get time handled while i get the clara-style things
> better understood"

The lab's debugging UX (post-hoc telemetry interrogation through a
wat-as-scripting-language surface) demanded `Time.now - 1.hour`-
shaped expressivity. Without a Duration type, every `since`
constraint was raw i64 nanos arithmetic. The arc closes that gap.

---

## What shipped

### Slice 1 — Duration variant + 7 unit constructors

`Value::Duration(i64)` runtime variant — non-negative nanoseconds
interval. Distinct from `Value::Instant` so the polymorphic
`:wat::time::-` (slice 2) can dispatch on the second argument's
tag. Arc 056 originally chose no separate Duration type; this
slice reverses that decision, with the doc comment on
`Value::Instant` updated to point at the reasoning.

Seven unit constructors at `:wat::time::*`:

```
:wat::time::Nanosecond  n   ; n
:wat::time::Microsecond n   ; n × 1_000
:wat::time::Millisecond n   ; n × 1_000_000
:wat::time::Second      n   ; n × 1_000_000_000
:wat::time::Minute      n   ; n × 60_000_000_000
:wat::time::Hour        n   ; n × 3_600_000_000_000
:wat::time::Day         n   ; n × 86_400_000_000_000
```

Each takes `:i64`, panics on negative input (durations are non-
negative; direction is in the operation, not the sign), panics on
i64 multiplication overflow with a diagnostic naming the unit's
max representable count (~290k years for Hour at i64::MAX nanos).

### Slice 2 — Polymorphic `:wat::time::-` and `:wat::time::+`

```
Instant - Duration -> Instant   (subtract interval)
Instant - Instant  -> Duration  (elapsed between)
Instant + Duration -> Instant   (advance by interval)
```

Same operator dispatches on the RHS Value variant tag. Runtime
matches at call time; type checker (`infer_polymorphic_time_arith`)
matches at expansion. Same flavor as ActiveSupport's
`time1 - time2 = duration` and `time - 1.hour = time`.

`Instant - Instant` panics if the result would be negative
(per §2: Durations are non-negative; subtract in the other
order). LHS-Duration is rejected — Duration arithmetic is
deferred until a real consumer demands it.

`chrono::Duration::nanoseconds(ns)` bridges i64 nanos to
`chrono::DateTime` arithmetic; `checked_add_signed` /
`checked_sub_signed` reject out-of-range results.

### Slice 3 — `ago` / `from-now` composers

```
(:wat::time::ago      duration) -> Instant   ; (- (now) duration)
(:wat::time::from-now duration) -> Instant   ; (+ (now) duration)
```

Convenience for the common case "relative to now." Direct
implementation against `Utc::now()` (consistent with the rest of
`:wat::time::*` being Rust-side; the BACKLOG considered wat-side
implementation but Rust-side won on consistency).

### Slice 4 — 14 pre-composed unit sugars

Seven units × `{ago, from-now}` = 14 helpers:

```
:wat::time::nanoseconds-ago   :wat::time::nanoseconds-from-now
:wat::time::microseconds-ago  :wat::time::microseconds-from-now
:wat::time::milliseconds-ago  :wat::time::milliseconds-from-now
:wat::time::seconds-ago       :wat::time::seconds-from-now
:wat::time::minutes-ago       :wat::time::minutes-from-now
:wat::time::hours-ago         :wat::time::hours-from-now
:wat::time::days-ago          :wat::time::days-from-now
```

Each takes `:i64`, applies the unit's nanos multiplier, computes
relative Instant via `Utc::now()` ± the offset. Same negativity +
overflow guards as the unit constructors. Reads cleaner at the
callsite than the `(ago (Hour 1))` decomposition:

```scheme
(:wat::time::hours-ago 1)            ; vs.
(:wat::time::ago (:wat::time::Hour 1))
```

---

## Tests

25 new wat-level deftests across `wat-tests/time.wat`:

- 10 covering each unit constructor + cross-unit equivalence
  (1 hour == 60 minutes; 1 day == 24 hours) + zero validity
- 6 covering `-` / `+` arithmetic — both dispatch arms,
  add-then-sub roundtrip, zero-Duration identity, Instant -
  itself
- 4 covering `ago` / `from-now` — direction sanity + zero-Duration
  identity
- 5 covering the unit sugars — sugar-vs-decomposition equivalence,
  forward/backward direction, zero-input identity

`cargo test --workspace --release`: all 728+ lib tests + every
integration test group green. Plus a pre-existing struct-to-form
nesting bug surfaced by arc 097's workspace test run (commit
`1f979d5`); fixed alongside in its own commit.

---

## What's NOT in this arc

- **Duration arithmetic.** `(Duration + Duration) -> Duration`,
  `(Duration - Duration) -> Duration` (latter requires Duration
  to remain non-negative; panic on would-be-negative results).
  Deferred until a real consumer demands. Users compose by
  constructing the duration they want directly.
- **`Instant - Instant -> Duration` for past-pointing pairs.** If
  `a < b`, `(- a b)` panics. Workaround: subtract in the other
  order. The substrate doesn't ship a `(time-between a b)` that
  takes order-agnostic input.
- **Wat-side stdlib version.** The composers + sugars could have
  shipped as one-line wat `define`s instead of Rust functions.
  Considered (BACKLOG sub-fog); chose Rust for consistency with
  the rest of `:wat::time::*`.
- **ISO8601 helper changes.** `:wat::time::from-iso8601` and
  `to-iso8601` ship as they were; this arc didn't touch them.

---

## Lessons

1. **Reversing arc 056's "no separate Duration type" decision.**
   The original choice — single Instant, "duration is two `now`
   calls and integer subtract" — was correct for arc 056's scope
   (basic timestamp primitives). When the lab pushed for
   ActiveSupport-shaped time math, the lack of a Duration tag made
   polymorphic `-` impossible — no way to distinguish
   `Instant - Duration` from `Instant - Instant`. Adding the tag
   unlocked the whole pattern. *Original design decisions get
   revisited when the use case shifts; the substrate is honest
   about its own design history in the doc comments.*

2. **Polymorphic dispatch via tag on Value variants.** Arc 050
   established the pattern (numeric arithmetic on i64 / f64);
   arc 097 extends it to time. The check-side
   (`infer_polymorphic_time_arith`) and runtime-side
   (`eval_time_sub`) dispatch the same way: match on the second
   argument's tag, return / produce the matching variant.

3. **The negative-Duration discipline ripples through the API.**
   Once §2 landed (Durations are non-negative; direction is in
   the operation), every operation that COULD produce a negative
   Duration had to either panic or refuse the input shape.
   `Instant - Instant` panics if `a < b`; LHS-Duration arithmetic
   is rejected in this slice. The discipline is consistent at
   every layer; the user doesn't have to track sign conventions
   to write correct code.

4. **Pre-existing test bracket bug surfaced under workspace
   test run.** Arc 091 slice 8's `struct-to-form.wat` had been
   shipping with `(:wat::test::run-ast (:wat::test::program …
   (:wat::core::vec :String)))` — the stdin arg accidentally
   nested inside `program`. Workspace tests caught it; chapter
   32's "no pre-existing-as-excuse" discipline meant fixing it
   alongside arc 097 rather than deflecting. *Visibility gaps
   close when a real workspace test surfaces them.*

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "we'll use iso8601 timestamps to nano ints - right?... Ruby
> Active's helpers for time are a wonderful ux... `(:wat::time/hours-ago some-time 1) -> (- some-time (:wat::time::Hour 1))`"

> "i figured it'd be time -> forms -> clara"

> "uh... let's get time handled while i get the clara-style things
> better understood"

> "1 - Hour ; 2 - interesting... i've never thought in negative
> terms - don't do this - that hurts my mind ; 4 - overflow of
> what?...."

> "3 - yeah - activesupport's time - time => duration - i think we
> should do that ; 5 - i only think in iso8601"

The arc closed when slice 5's INSCRIPTION shipped + `cargo test
--workspace --release` came back green for the fourth time in a
row. The substrate is what the user said it should be when he
named it.

**PERSEVERARE.**
