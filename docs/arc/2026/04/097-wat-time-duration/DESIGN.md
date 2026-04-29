# Arc 097 — `:wat::time::Duration` + arithmetic + ActiveSupport helpers — DESIGN

**Status:** READY — opened and settled 2026-04-29. All five
open questions resolved by Q&A. Ready to ship slice 1.

The decisions:
- **§1** Bare names in `:wat::time::*` (Hour, Minute, etc.); no `Duration::` prefix.
- **§2** Duration is non-negative; constructors panic on negative input. Direction lives in operations (`-`, `+`, `ago`, `from-now`).
- **§3** Polymorphic `:wat::time::-` — Instant - Instant returns Duration (ActiveSupport-shaped).
- **§4** Constructors panic on i64 overflow with diagnostic.
- **§5** Existing ISO8601 helpers untouched.

Duration is a runtime `Value::Duration(i64)` variant — not a typealias — so the polymorphic `-` can dispatch.

**Predecessor of:** [arc 093](../093-wat-telemetry-workquery/DESIGN.md)
slice 2 (the `Since` / `Until` constraint variants take an `Instant`;
the worked examples assume `(hours-ago 1)` reads cleanly).

**Reusable beyond telemetry.** Any wat program that does time
arithmetic — proofs, lab tests, scheduled work, debug scripts —
inherits these helpers.

---

## What's missing

Currently shipped surface in `:wat::time::*`:

```
:wat::time::now              ; -> Instant
:wat::time::epoch-nanos      ; Instant -> i64
:wat::time::epoch-millis     ; Instant -> i64
:wat::time::epoch-seconds    ; Instant -> i64
:wat::time::at-nanos         ; i64 -> Instant
:wat::time::at-millis        ; i64 -> Instant
:wat::time::at               ; ? (epoch-seconds?)
:wat::time::from-iso8601     ; String -> Instant (or Result/Option)
:wat::time::to-iso8601       ; Instant -> String
```

Conversion primitives only. No Duration. No arithmetic. No
"X ago" composers. Without these, time math forces the user to
write through nanos:

```scheme
;; The status quo — UX nobody loves
(:wat::time::at-nanos
  (:wat::core::-
    (:wat::time::epoch-nanos (:wat::time::now))
    (:wat::core::* 3600 1000000000)))
```

That's "one hour ago." Six function calls; explicit nanos
arithmetic. Every callsite repeats the same pattern.

The user's framing (2026-04-29):

> *"Ruby Active's helpers for time are a wonderful ux...
> some_timestamp - 1.hour # i find this so intituive to work
> with..."*

ActiveSupport gets this right. We adopt the same shape.

## What ships

### `:wat::time::Duration`

A new runtime `Value::Duration(i64)` variant — distinct from
`Value::Instant` (already shipped per arc 056). Both are i64
nanos under the hood, but the type tags let the substrate
dispatch on which is which. NOT a typealias to `:i64` — that
would lose the type discrimination needed for polymorphic `-`
(see §3).

```
:wat::time::Duration   ; Value::Duration(i64), non-negative nanos interval
:wat::time::Instant    ; Value::Instant(i64), nanos since epoch (shipped)
```

Per §2: Duration is always non-negative. Constructors panic
on negative input.

### Constructors (i64 → Duration)

ActiveSupport's `1.hour` shape. Each constructor takes an `:i64`
count and returns a Duration (multiplied by the unit's nanos):

```
(:wat::time::Nanosecond  n)   ; n
(:wat::time::Microsecond n)   ; n × 1_000
(:wat::time::Millisecond n)   ; n × 1_000_000
(:wat::time::Second      n)   ; n × 1_000_000_000
(:wat::time::Minute      n)   ; n × 60_000_000_000
(:wat::time::Hour        n)   ; n × 3_600_000_000_000
(:wat::time::Day         n)   ; n × 86_400_000_000_000
```

PascalCase per arc 048's enum-naming convention (and convention
for "value constructors that read as the unit"). Example usage:
`(:wat::time::Hour 1)` reads as "1 Hour."

### Arithmetic — polymorphic on type tags

```
(:wat::time::- instant duration)   ; -> Instant   (subtract interval)
(:wat::time::- instant_a instant_b) ; -> Duration (elapsed between)
(:wat::time::+ instant duration)   ; -> Instant   (advance by interval)
```

Polymorphic via runtime dispatch on the second argument's tag:

| LHS | RHS | Result |
|---|---|---|
| Instant | Duration | Instant |
| Instant | Instant | Duration |
| Instant | Duration (via `+`) | Instant |

Same surface as ActiveSupport: `time1 - time2 = duration` and
`time - 1.hour = time`. The runtime checks the RHS variant and
picks the right behavior.

`Duration - Duration` and `Duration + Duration` not in this
arc — defer until a real consumer demands. Users can compose by
constructing the duration they want directly (`(Hour 1)`,
`(Minute 30)`).

`Instant - Instant -> Duration` always returns a non-negative
Duration: if `instant_a < instant_b`, the substrate panics with
`"Duration would be negative; subtract in the other order"`.
Per §2 — Durations are non-negative; the operation enforces it.

### Composers — "X ago" / "X from now"

```
(:wat::time::ago      duration)   ; -> Instant   (- (now) duration)
(:wat::time::from-now duration)   ; -> Instant   (+ (now) duration)
```

Convenience for the common case "relative to now."

### Sugar — pre-composed unit-ago / from-now

```
(:wat::time::nanoseconds-ago  n)
(:wat::time::microseconds-ago n)
(:wat::time::milliseconds-ago n)
(:wat::time::seconds-ago      n)
(:wat::time::minutes-ago      n)
(:wat::time::hours-ago        n)
(:wat::time::days-ago         n)

(:wat::time::nanoseconds-from-now  n)
(:wat::time::microseconds-from-now n)
(:wat::time::milliseconds-from-now n)
(:wat::time::seconds-from-now      n)
(:wat::time::minutes-from-now      n)
(:wat::time::hours-from-now        n)
(:wat::time::days-from-now         n)
```

Each is `(ago (Hour n))` etc. — a one-line `define`. Ship them
because the script ergonomics matter: `(hours-ago 1)` reads
better than `(:wat::time::ago (:wat::time::Hour 1))` at every
callsite.

## Worked examples

Ruby's pattern:

```ruby
1.hour                # ActiveSupport::Duration
some_time - 1.hour    # Time
1.hour.ago            # Time
2.days.from_now       # Time
```

Wat equivalents:

```scheme
;; "1.hour" — duration value
(:wat::time::Hour 1)

;; "some_time - 1.hour"
(:wat::time::- some-time (:wat::time::Hour 1))

;; "1.hour.ago"
(:wat::time::hours-ago 1)
;; or, equivalently:
(:wat::time::ago (:wat::time::Hour 1))

;; "2.days.from_now"
(:wat::time::days-from-now 2)

;; ISO8601 — absolute
(:wat::time::from-iso8601 "2026-04-29T10:00:00Z")
```

## Slice plan

**Slice 1** — Duration runtime variant + unit constructors (7).
- New `Value::Duration(i64)` runtime variant — sibling to
  `Value::Instant(i64)` (arc 056). Type tag in the substrate's
  type table; `:wat::time::Duration` resolves to it.
- 7 unit constructors (`Nanosecond`, `Microsecond`, `Millisecond`,
  `Second`, `Minute`, `Hour`, `Day`) — each takes `:i64`,
  panics on `n < 0` (per §2), panics on overflow (per §4),
  multiplies by unit's nanos, returns `Value::Duration`.
- Rust-side: ~7 functions; type registration; one new variant
  in the runtime's `Value` enum + matching cases in
  `type_name`, equality, EDN write, etc.
- Tests: each constructor; negative input panics with §2's
  diagnostic; overflow input panics with §4's diagnostic; valid
  inputs produce expected nanos.

**Slice 2** — Instant ± Duration arithmetic.
- `:wat::time::-` and `:wat::time::+`.
- Rust-side: type-checked addition/subtraction.
- Tests: verify type discipline; verify wrap/overflow safety.

**Slice 3** — `ago` / `from-now` composers.
- `:wat::time::ago duration -> Instant`.
- `:wat::time::from-now duration -> Instant`.
- Each one-line over `(now)` + arithmetic from slice 2.
- Tests.

**Slice 4** — Pre-composed unit-ago / unit-from-now (14 helpers).
- All 14 helpers as one-line wat (or rust) defines on top of
  slice 3.
- Tests verify the composition holds across the unit table.

**Slice 5** — INSCRIPTION + USER-GUIDE update + arc 093 dependency
note resolution.

## Open questions

### §1. Where do constructors live — SETTLED

**Settled.** Bare names in `:wat::time::*`: `:wat::time::Hour`,
`:wat::time::Minute`, etc. PascalCase signals the value-
constructor role; no `:wat::time::Duration::` prefix.

Per the user (2026-04-29): *"1 - Hour."*

### §2. Negative durations — SETTLED (rejected)

**Settled.** Duration is **always non-negative**. An interval,
not a vector. Direction is expressed by the OPERATION, never by
the sign of the duration.

Per the user (2026-04-29):

> *"interesting... i've never thought in negative terms - don't
> do this - that hurts my mind"*

The Ruby/ActiveSupport idiom is the model: `1.hour.ago` is
positive; the verb (`.ago` / `.from_now`) determines direction.
Wat adopts the same:

- `(:wat::time::- instant duration)` — go backward by duration.
- `(:wat::time::+ instant duration)` — go forward by duration.
- `(:wat::time::ago duration)` — `(- (now) duration)`.
- `(:wat::time::from-now duration)` — `(+ (now) duration)`.

**Negative input to constructors panics** with a clear
diagnostic: `(Hour -1)` is invalid; the substrate refuses it at
construction time rather than letting a negative value flow
downstream where it would silently invert the meaning of every
arithmetic operation.

Slice 1 implementation: every constructor checks `n >= 0`;
panic with `"(Hour -1): Duration must be non-negative; use
hours-from-now / from-now to express future intervals or
hours-ago / ago for past intervals"`.

### §3. Instant - Instant -> Duration? — SETTLED (yes)

**Settled.** `:wat::time::-` is polymorphic on the RHS variant
tag — Instant - Instant returns Duration; Instant - Duration
returns Instant. Same shape as ActiveSupport's `time1 - time2`
returning the elapsed interval.

Per the user (2026-04-29):

> *"yeah - activesupport's time - time => duration - i think we
> should do that"*

The dispatch is mechanical: at runtime, `:wat::time::-` checks
the RHS's Value variant. `Value::Instant` second arg → return
`Value::Duration` (the difference). `Value::Duration` second arg
→ return `Value::Instant` (the subtracted moment). Two arms; one
operator; reads exactly like the math.

Per §2 (Duration always non-negative): if `(- a b)` would
produce a negative interval (a is before b), panic with
`"Duration would be negative; subtract in the other order"`.
The discipline propagates — Durations are intervals, not
vectors, anywhere they appear.

Slice 2 implementation: `:wat::time::-` runtime dispatch
matches on RHS Value variant; check the LHS is `Value::Instant`
in both arms (not allowed to subtract Duration from anything —
no Duration arithmetic in this arc).

### §4. Overflow handling — SETTLED (panic)

**Settled.** Constructors panic on overflow with a clear
diagnostic. Same shape as the negative-input panic (§2).

**What overflows, concretely.** `(Hour n)` multiplies `n ×
3,600,000,000,000` (one hour in nanoseconds). i64::MAX is
~9.2 × 10^18. Max hours that fit: ~2.5 million (~290,000
years). For any realistic interval (seconds / minutes / hours /
days / years / centuries) the multiplication is nowhere near
overflow.

The case where it matters: someone mistypes a constant or
computes a value programmatically that produces a number larger
than makes sense. `(Hour 9_999_999_999_999)` overflows i64 in
the multiplication; without a check, the result wraps to a
nonsense value (a year ago becomes next Tuesday); with a check,
the substrate refuses with `"(Hour 9.99e12) overflows
representable Duration (~290k years max)"`.

**Why panic, not saturate or wrap.**
- **Wrap** is mathematically incoherent for time. Wrapping
  modular arithmetic on seconds-of-the-day produces wrong-by-
  default results.
- **Saturate** stays in range but silently degrades — `(Day
  10^15)` becomes `i64::MAX` nanos with no diagnostic.
  Probably nothing the user wants.
- **Panic** is honest. The constraint is real (i64 has a
  range); the user typed something outside it; tell them.

The check is one branch in the constructor. Free in practice
because nobody hits it; load-bearing only when someone's
`(Day 999_999_999_999)` makes them re-read what they wrote.

### §5. ISO8601 helpers — SETTLED (don't change in this arc)

**Settled.** Existing `from-iso8601` and `to-iso8601` ship as
they are; this arc doesn't touch them.

Per the user (2026-04-29): *"i only think in iso8601."*

The user's mental model is "absolute timestamps as ISO8601
strings; relative as `(hours-ago N)`." Both surfaces are present
already — `from-iso8601` for absolute input, the arc 097 sugar
for relative. No new ISO8601 work needed here.

If a future consumer hits a UX issue with the existing helpers'
return type or precision, separate arc.

---

## Predecessors / dependencies

**Shipped:**
- `:wat::time::*` conversion primitives (now, epoch-*, at-*,
  to/from-iso8601).
- arc 048 — user-defined enums (used if Duration becomes a struct
  rather than typealias; not currently planned).
- arc 057 — typealias system.

**Depends on:** nothing else. Pure substrate add.

## What this enables

- Arc 093 slice 2's `Since(Instant)` / `Until(Instant)` constraints
  read cleanly.
- Lab proofs with time-window logic stop computing nanos arithmetic
  by hand.
- Future scheduling primitives (cron-style) have a sensible
  duration substrate.
- General wat scripts gain ergonomic time math.

**PERSEVERARE.**
