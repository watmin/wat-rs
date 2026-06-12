# Strike — the `Duration` readout family (time-ops enrichment)

> **SHIPPED + RENAMED.** This brief proposed the `as-<unit>` spelling; an intueri
> cast (2026-06-11) ratified the family but renamed it to **bare unit-plural**
> (`nanoseconds` … `days`) — capitalized `Second` constructs, lowercase `seconds`
> reads. The `as-<unit>` names below are the original proposal, kept as the
> strike's record; the shipped surface is the bare names.

> Born of a reach-stumble while drawing arc 259 S2c-iii: the timing correction
> reached for "read this `Duration` out as a number" and the tool was absent.
> `feedback_reach_stumble_is_the_signal` — the reach IS the spec; pivot and make
> the tool, first-class, never a workaround. S2c-iii then stands on it.

## The gap (one sentence)

`Duration` is a **write-only value**: seven unit constructors build one from an
i64 (`Nanosecond` … `Day`), arithmetic and comparison work on it — but there is
**no verb to read the number back out** in any unit. `Instant` round-trips
(`at-millis` ↔ `epoch-millis`); `Duration` has only the IN half.

## The fix — the symmetric readout family

Seven constructors deserve seven readouts. Mint `:wat::time::Duration -> :i64`
readouts, one per unit, **truncating toward zero** exactly like `epoch-millis`
(integer divide the stored nanos by the unit's nanos-per-unit constant):

```
as-nanoseconds · as-microseconds · as-milliseconds · as-seconds ·
as-minutes · as-hours · as-days
```

**Contract decision (pinned):** each takes one `:wat::time::Duration`, returns
`:i64`, truncating (floor for the non-negative Durations the type guarantees).
No rounding, no panic — a `Value::Duration(i64 nanos)` divided by a positive
constant cannot overflow or error. **Names are proposed, not final** — an
intueri cast ratifies the spelling before commit (the FAMILY is locked by
symmetry; only the surface spelling is open).

## Rooms — read in order (every pattern is already on disk; mirror it)

1. `src/time.rs:295–348` — the `NANOS_PER_*` constants + `unit_constructor`
   helper + the seven `eval_time_unit_*` thin wrappers. **Mirror this exactly**:
   add a `unit_readout(op, unit_nanos, args, …)` helper (require ONE
   `Value::Duration(nanos)` arg → return `Value::i64(nanos / unit_nanos)`), then
   seven thin `eval_time_as_*` wrappers each calling it with the right constant.
   `NANOS_PER_MICRO` does not exist yet — add it (`1_000`) beside the others.
2. `src/runtime.rs:4616–4622` — the seven unit-constructor dispatch arms. Add
   seven sibling arms `":wat::time::as-<unit>" => crate::time::eval_time_as_<unit>(…)`
   right after them (same `.map_err(Into::into)` shape).
3. `src/check.rs:16499–16517` — the constructor sig loop (`i64 -> Duration`). Add
   a sibling loop right after it registering the seven readout names
   `Duration -> i64` (params `vec![duration_ty()]`, ret `i64_ty()`).
4. `src/time.rs:1–31` — the module-doc verb table. Add the seven readouts to it
   (under the `epoch-*` block — they are the `Duration` analogues).

## Sketch (the helper — fill in the rest by mirroring `unit_constructor`)

```rust
fn unit_readout(
    op: &'static str, unit_nanos: i64, args: &[WatAST],
    list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 { /* ArityMismatch, exactly like unit_constructor */ }
    match eval(&args[0], env, sym)? {
        Value::Duration(nanos) => Ok(Value::i64(nanos / unit_nanos)),
        other => /* TypeMismatch: op expects :wat::time::Duration, got <other> */,
    }
}
/// `(:wat::time::as-milliseconds d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_as_millisecond(...) -> ... {
    unit_readout(":wat::time::as-milliseconds", NANOS_PER_MILLI, args, list_span, env, sym)
}
// … six siblings: as-nanoseconds(1) as-microseconds(NANOS_PER_MICRO)
//   as-seconds(NANOS_PER_SECOND) as-minutes(NANOS_PER_MINUTE)
//   as-hours(NANOS_PER_HOUR) as-days(NANOS_PER_DAY)
```

Match `unit_constructor`'s error idioms (require_i64's sibling for Duration; reuse
the existing TypeMismatch/ArityMismatch shapes — grep how a Duration-consuming
verb like `eval_time_ago` rejects a non-Duration arg, and mirror it).

## Blast radius

`src/time.rs` (+ helper + 7 wrappers + 1 const + doc), `src/runtime.rs` (+7
dispatch arms), `src/check.rs` (+1 sig loop). **No new types, no Value variant,
no changes to any existing verb.** Purely additive.

## STOP triggers (reject, do not work around)

- **STOP-1:** if `Value::Duration`'s inner repr is NOT a plain i64 nanosecond
  count (verify at `src/runtime.rs` `Value::Duration(`), STOP — the division
  premise is wrong; surface it.
- **STOP-2:** if adding the sig loop makes any EXISTING time test go red (a name
  collision with a constructor), STOP and report — the readout names must not
  shadow anything.

## Gate (the load-bearing proof — run it yourself)

```
cargo test --release -p wat --test nursery probe_time_duration_readout   # 4/4 GREEN (RED at HEAD)
cargo test --release -p wat --test nursery                               # no NEW reds (known: 4 — arc-255 reflection ×2 + undefined-builtin ×2)
cargo build --release                                                    # clean
```

Expected: probe 4/4 green; nursery serial `855 passed / 4 failed` (the 4 known
pre-existing). Runtime ~5–10 min.
