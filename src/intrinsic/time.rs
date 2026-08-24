//! `:wat::time::` intrinsics — arc 255 home #2, carved to the
//! `#[wat_intrinsic]` form (255.1c-time). Bodies MOVED from `src/time.rs`
//! (arc 056 / arc 097), which now keeps only the two `pub fn`s that are not
//! part of the `:wat::time::*` dispatch surface (`process_boot_instant` /
//! `set_process_boot_instant`, used by `freeze.rs` / `kernel/spawn.rs` /
//! `distribution/mod.rs`).
//!
//! **Lineage: Java / Clojure.** Single `Instant` covers both "when did this
//! happen?" and "how long did this take?" — no separate monotonic type.
//! UTC only. ISO 8601 / RFC 3339 round-trips. `Instant` backs
//! `chrono::DateTime<chrono::Utc>`; `Duration` backs non-negative i64 nanos.
//!
//! ## The split this home exists to prove (255.1c-time's load-bearing point)
//!
//! Every `core::Bytes` entry (home #1) is `Pure` + `Deterministic` — a home
//! whose every row takes the same two values on the purity/determinism axes
//! cannot falsify the metadata contract. `:wat::time::` is the first home
//! that carries a genuinely `Nondeterministic` row, **classified from each
//! handler's actual body**, not from a naming convention:
//!
//! - **Reads the wall clock** (`Utc::now()` appears in the body) →
//!   `Nondeterministic`: `now`, `ago`, `from-now`, and the 14 pre-composed
//!   `<unit>-ago` / `<unit>-from-now` sugars. 17 rows.
//! - **Pure function of its argument(s) alone** (no clock access) →
//!   `Deterministic`: `at`/`at-millis`/`at-nanos`, `from-iso8601`/
//!   `to-iso8601`, `epoch-seconds`/`epoch-millis`/`epoch-nanos`, the seven
//!   unit constructors, the seven unit readouts, and `+`/`-`. 24 rows.
//!
//! `epoch-seconds`/`epoch-millis`/`epoch-nanos` are the delta worth naming:
//! an orientation sketch guessed these were clock reads (they take an
//! `Instant`, so it *sounds* time-related), but the body only reads fields
//! off an already-sampled `Instant` argument — no `Utc::now()` anywhere.
//! They are `Deterministic`. The body is truth; the sketch was orientation.
//!
//! `@Category` is a closed, append-only domain; `Clock` (renamed `Entropic`,
//! 2026-08-19 — 299.3-entropic: it named the DEVICE, not the doing; `time::now`
//! and `Uuid/v4` are the same category) and `Arithmetic` were added for this
//! family (2026-08-15). Every representation-transforming row here is a
//! representation transform (raw i64 ⇄ Instant/Duration, String ⇄ Instant, or
//! Instant/Duration combined into a new Instant/Duration) → `Transform`, the
//! same bucket `core::Bytes::to-hex`/`from-hex` and the doc-contract's own
//! "Blend two things" fixture (`crates/wat-doc/src/lib.rs:993`, a plain 2-arg
//! pure combinator) occupy. Every `Nondeterministic` row samples the wall
//! clock → `Entropic`, matching the `Uuid/v4` precedent in the same fixture
//! file (`crates/wat-doc/src/lib.rs:985`: nondeterministic + `Reflection`).

use std::sync::Arc;

use chrono::{DateTime, SecondsFormat, TimeZone, Utc};
use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, TrackedValue, Value,
    ValueSnapshot,
};

// ─── Constructors ────────────────────────────────────────────────────

/// Current wall-clock time.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @ret     :wat::time::Instant the instant sampled at call time
/// @example-norun (:wat::time::now) #=> #inst "2026-08-13T12:00:00.000000000Z"
#[wat_intrinsic(":wat::time::now")]
pub(crate) fn eval_time_now(
    _env: &Environment, // rune:lint(unused-env) — `now` samples the clock only; no bindings needed
    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
    _span: &Span, // rune:lint(unused-span) — `now` is 0-arity; the shim's own arity check owns that failure mode
) -> Result<Value, EvalBreak> {
    Ok(Value::Instant(Utc::now()))
}

/// Builds an Instant from integer seconds since 1970-01-01T00:00:00Z (the
/// Unix epoch). Negative values are pre-epoch and behave per chrono.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     secs :wat::core::i64 epoch seconds since the Unix epoch (may be negative)
/// @ret     :wat::time::Instant the instant at that epoch-seconds mark
/// @example (:wat::time::epoch-seconds (:wat::time::at 1000000000)) #=> 1000000000
/// @see     :wat::time::epoch-seconds
#[wat_intrinsic(":wat::time::at")]
pub(crate) fn eval_time_at(
    secs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::at";
    let secs = require_i64(OP, eval(secs, env, sym)?, list_span)?;
    let dt = Utc.timestamp_opt(secs, 0).single().ok_or_else(|| {
        // chrono range error — secs is a plain i64 from an evaluated Value; list_span is the best available location
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "epoch-seconds in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range i64"))
        })
    })?;
    Ok(Value::Instant(dt))
}

/// Builds an Instant from integer milliseconds since the Unix epoch.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     ms :wat::core::i64 epoch milliseconds since the Unix epoch (may be negative)
/// @ret     :wat::time::Instant the instant at that epoch-millis mark
/// @example (:wat::time::epoch-millis (:wat::time::at-millis 1000000000000)) #=> 1000000000000
/// @see     :wat::time::epoch-millis
#[wat_intrinsic(":wat::time::at-millis")]
pub(crate) fn eval_time_at_millis(
    ms: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::at-millis";
    let ms = require_i64(OP, eval(ms, env, sym)?, list_span)?;
    let dt = Utc.timestamp_millis_opt(ms).single().ok_or_else(|| {
        // chrono range error — ms is a plain i64 from an evaluated Value; list_span is the best available location
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "epoch-ms in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range i64"))
        })
    })?;
    Ok(Value::Instant(dt))
}

/// Builds an Instant from integer nanoseconds since the Unix epoch. i64 ns
/// saturates at year ~2262.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     ns :wat::core::i64 epoch nanoseconds since the Unix epoch (may be negative)
/// @ret     :wat::time::Instant the instant at that epoch-nanos mark
/// @example (:wat::time::epoch-nanos (:wat::time::at-nanos 1000000000000000000)) #=> 1000000000000000000
/// @see     :wat::time::epoch-nanos
#[wat_intrinsic(":wat::time::at-nanos")]
pub(crate) fn eval_time_at_nanos(
    ns: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::at-nanos";
    let ns = require_i64(OP, eval(ns, env, sym)?, list_span)?;
    Ok(Value::Instant(Utc.timestamp_nanos(ns)))
}

/// Parses an ISO 8601 / RFC 3339 string into an Instant. `:None` on parse
/// failure. Accepts `parse_from_rfc3339` grammar (the practical ISO 8601
/// subset).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     s :wat::core::String the ISO 8601 / RFC 3339 timestamp string
/// @ret     (:wat::core::Option :- [:wat::time::Instant]) Some(Instant) on success, None on malformed input
/// @example (:wat::time::from-iso8601 "not-a-date") #=> :None
/// @see     :wat::time::to-iso8601
#[wat_intrinsic(":wat::time::from-iso8601")]
pub(crate) fn eval_time_from_iso8601(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::from-iso8601";
    let s = require_string(OP, eval(s, env, sym)?, list_span)?;
    let parsed = DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));
    let inner = parsed.map(Value::Instant);
    Ok(Value::Option(Arc::new(inner)))
}

// ─── Formatter ───────────────────────────────────────────────────────

/// Formats an Instant as ISO 8601 / RFC 3339 with N fractional second
/// digits. `digits` is clamped to `[0, 9]`; output always UTC (`Z` suffix).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     inst :wat::time::Instant the instant to format
/// @arg     digits :wat::core::i64 fractional-second digit count, clamped to [0, 9]
/// @ret     :wat::core::String the ISO 8601 / RFC 3339 string, `Z`-suffixed
/// @example (:wat::time::to-iso8601 (:wat::time::at 0) 0) #=> "1970-01-01T00:00:00Z"
/// @see     :wat::time::from-iso8601
#[wat_intrinsic(":wat::time::to-iso8601")]
pub(crate) fn eval_time_to_iso8601(
    inst: &WatAST,
    digits: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::to-iso8601";
    let inst = require_instant(OP, eval(inst, env, sym)?, list_span)?;
    let digits_raw = require_i64(OP, eval(digits, env, sym)?, list_span)?;
    let digits = digits_raw.clamp(0, 9) as u32;
    let formatted = if digits == 0 {
        // SecondsFormat::Secs already drops the fractional part and
        // uses a Z suffix — exactly what we want at digits=0.
        inst.to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        // Hand-format: integer datetime + . + N digits + Z. chrono's
        // built-in fractional formatters round to 3/6/9 only, but our
        // contract supports every digit count in [0, 9].
        let secs_part = inst.format("%Y-%m-%dT%H:%M:%S");
        let nanos = inst.timestamp_subsec_nanos();
        let scaled = nanos / 10_u32.pow(9 - digits);
        format!(
            "{}.{:0>width$}Z",
            secs_part,
            scaled,
            width = digits as usize
        )
    };
    Ok(Value::String(Arc::new(formatted)))
}

// ─── Accessors ───────────────────────────────────────────────────────

/// Reads epoch seconds off an Instant. Truncating; sub-second precision
/// lost.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     inst :wat::time::Instant the instant to read
/// @ret     :wat::core::i64 epoch seconds since the Unix epoch, truncated
/// @example (:wat::time::epoch-seconds (:wat::time::at 1000000000)) #=> 1000000000
/// @see     :wat::time::at
#[wat_intrinsic(":wat::time::epoch-seconds")]
pub(crate) fn eval_time_epoch_seconds(
    inst: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::epoch-seconds";
    let inst = require_instant(OP, eval(inst, env, sym)?, list_span)?;
    Ok(Value::i64(inst.timestamp()))
}

/// Reads epoch milliseconds off an Instant. Truncating to ms.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     inst :wat::time::Instant the instant to read
/// @ret     :wat::core::i64 epoch milliseconds since the Unix epoch, truncated
/// @example (:wat::time::epoch-millis (:wat::time::at-millis 1000000000000)) #=> 1000000000000
/// @see     :wat::time::at-millis
#[wat_intrinsic(":wat::time::epoch-millis")]
pub(crate) fn eval_time_epoch_millis(
    inst: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::epoch-millis";
    let inst = require_instant(OP, eval(inst, env, sym)?, list_span)?;
    Ok(Value::i64(inst.timestamp_millis()))
}

/// Reads epoch nanoseconds off an Instant. Panics if the instant is outside
/// i64-nanosecond representable range (i.e., before ~1677 or after ~2262).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     inst :wat::time::Instant the instant to read
/// @ret     :wat::core::i64 epoch nanoseconds since the Unix epoch
/// @example (:wat::time::epoch-nanos (:wat::time::at-nanos 1000000000000000000)) #=> 1000000000000000000
/// @see     :wat::time::at-nanos
#[wat_intrinsic(":wat::time::epoch-nanos")]
pub(crate) fn eval_time_epoch_nanos(
    inst: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::epoch-nanos";
    let inst = require_instant(OP, eval(inst, env, sym)?, list_span)?;
    let ns = inst.timestamp_nanos_opt().ok_or_else(|| {
        // chrono range error — inst is a plain DateTime from an evaluated Value; list_span is the best available location
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "instant in i64-nanosecond range (~1677 to ~2262)",
            got: Box::new(ValueSnapshot::unavailable("out-of-range instant"))
        })
    })?;
    Ok(Value::i64(ns))
}

// ─── Arc 097 — Duration constructors ────────────────────────────────
//
// Seven unit constructors at `:wat::time::*` (Nanosecond, Microsecond,
// Millisecond, Second, Minute, Hour, Day). Each takes `:i64`, panics
// on negative input (durations are non-negative; direction lives in
// the operation, not the sign), panics on i64 multiplication overflow
// (~290k years for Hour at i64::MAX nanos; nobody hits it; check is
// free; clear error when someone mistypes a constant).
//
// The shared `unit_constructor` helper does the type check, negativity
// check, overflow-on-multiply check (arity is the macro shim's job now);
// the seven public functions just thread their unit's nanos-per-unit
// constant.

const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MILLI: i64 = 1_000_000;
const NANOS_PER_SECOND: i64 = 1_000_000_000;
const NANOS_PER_MINUTE: i64 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: i64 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: i64 = 24 * NANOS_PER_HOUR;

fn unit_constructor(
    op: &'static str,
    unit_nanos: i64,
    n: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let n = require_i64(op, eval(n, env, sym)?, list_span)?;
    if n < 0 {
        panic!(
            "({} {}): Duration must be non-negative; use ago / from-now \
             helpers (or :wat::time::- on Instants) to express past or \
             future intervals — direction lives in the operation, not \
             the sign of the duration",
            op, n
        );
    }
    let nanos = n.checked_mul(unit_nanos).unwrap_or_else(|| {
        panic!(
            "({} {}): overflows representable Duration; i64 nanos \
             saturates at ~9.2e18, so unit constants larger than \
             {} are out of range (e.g. Hour caps at ~2.5M; ~290k \
             years)",
            op,
            n,
            i64::MAX / unit_nanos
        )
    });
    Ok(Value::Duration(nanos))
}

/// Builds a Duration of N nanoseconds. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of nanoseconds (non-negative)
/// @ret     :wat::time::Duration the Duration of N nanoseconds
/// @example (:wat::time::nanoseconds (:wat::time::Nanosecond 5)) #=> 5
/// @see     :wat::time::nanoseconds
#[wat_intrinsic(":wat::time::Nanosecond")]
pub(crate) fn eval_time_unit_nanosecond(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Nanosecond", 1, n, list_span, env, sym).map_err(Into::into)
}

/// Builds a Duration of N microseconds. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of microseconds (non-negative)
/// @ret     :wat::time::Duration the Duration of N microseconds
/// @example (:wat::time::microseconds (:wat::time::Microsecond 5)) #=> 5
/// @see     :wat::time::microseconds
#[wat_intrinsic(":wat::time::Microsecond")]
pub(crate) fn eval_time_unit_microsecond(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Microsecond", NANOS_PER_MICRO, n, list_span, env, sym)
        .map_err(Into::into)
}

/// Builds a Duration of N milliseconds. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of milliseconds (non-negative)
/// @ret     :wat::time::Duration the Duration of N milliseconds
/// @example (:wat::time::milliseconds (:wat::time::Millisecond 5)) #=> 5
/// @see     :wat::time::milliseconds
#[wat_intrinsic(":wat::time::Millisecond")]
pub(crate) fn eval_time_unit_millisecond(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Millisecond", NANOS_PER_MILLI, n, list_span, env, sym)
        .map_err(Into::into)
}

/// Builds a Duration of N seconds. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of seconds (non-negative)
/// @ret     :wat::time::Duration the Duration of N seconds
/// @example (:wat::time::seconds (:wat::time::Second 5)) #=> 5
/// @see     :wat::time::seconds
#[wat_intrinsic(":wat::time::Second")]
pub(crate) fn eval_time_unit_second(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Second", NANOS_PER_SECOND, n, list_span, env, sym).map_err(Into::into)
}

/// Builds a Duration of N minutes. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of minutes (non-negative)
/// @ret     :wat::time::Duration the Duration of N minutes
/// @example (:wat::time::minutes (:wat::time::Minute 5)) #=> 5
/// @see     :wat::time::minutes
#[wat_intrinsic(":wat::time::Minute")]
pub(crate) fn eval_time_unit_minute(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Minute", NANOS_PER_MINUTE, n, list_span, env, sym).map_err(Into::into)
}

/// Builds a Duration of N hours. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of hours (non-negative)
/// @ret     :wat::time::Duration the Duration of N hours
/// @example (:wat::time::hours (:wat::time::Hour 5)) #=> 5
/// @see     :wat::time::hours
#[wat_intrinsic(":wat::time::Hour")]
pub(crate) fn eval_time_unit_hour(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Hour", NANOS_PER_HOUR, n, list_span, env, sym).map_err(Into::into)
}

/// Builds a Duration of N days. Panics on negative N or overflow.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     n :wat::core::i64 the count of days (non-negative)
/// @ret     :wat::time::Duration the Duration of N days
/// @example (:wat::time::days (:wat::time::Day 5)) #=> 5
/// @see     :wat::time::days
#[wat_intrinsic(":wat::time::Day")]
pub(crate) fn eval_time_unit_day(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_constructor(":wat::time::Day", NANOS_PER_DAY, n, list_span, env, sym).map_err(Into::into)
}

// ─── Duration readout family — symmetric OUT half of the constructors ──
//
// Seven `:wat::time::<unit>` verbs (bare unit-plural) mirror the seven
// constructors: capitalized `Second` constructs a Duration, lowercase-plural
// `seconds` reads one out. Each takes a `:wat::time::Duration` and returns
// `:i64` by dividing the stored nanosecond count by the unit's nanos-per-unit
// constant, truncating toward zero (same behaviour as `epoch-millis`).
//
// The shared `unit_readout` helper does the type check and division (arity
// is the macro shim's job now); the seven public functions just thread
// their constant.

fn unit_readout(
    op: &'static str,
    unit_nanos: i64,
    d: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let ns = require_duration(op, eval(d, env, sym)?, list_span)?;
    Ok(Value::i64(ns / unit_nanos))
}

/// Reads a Duration's length in nanoseconds. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in nanoseconds, truncated
/// @example (:wat::time::nanoseconds (:wat::time::Nanosecond 5)) #=> 5
/// @see     :wat::time::Nanosecond
#[wat_intrinsic(":wat::time::nanoseconds")]
pub(crate) fn eval_time_nanoseconds(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::nanoseconds", 1, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in microseconds. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in microseconds, truncated
/// @example (:wat::time::microseconds (:wat::time::Microsecond 5)) #=> 5
/// @see     :wat::time::Microsecond
#[wat_intrinsic(":wat::time::microseconds")]
pub(crate) fn eval_time_microseconds(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::microseconds", NANOS_PER_MICRO, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in milliseconds. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in milliseconds, truncated
/// @example (:wat::time::milliseconds (:wat::time::Millisecond 5)) #=> 5
/// @see     :wat::time::Millisecond
#[wat_intrinsic(":wat::time::milliseconds")]
pub(crate) fn eval_time_milliseconds(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::milliseconds", NANOS_PER_MILLI, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in seconds. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in seconds, truncated
/// @example (:wat::time::seconds (:wat::time::Second 5)) #=> 5
/// @see     :wat::time::Second
#[wat_intrinsic(":wat::time::seconds")]
pub(crate) fn eval_time_seconds(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::seconds", NANOS_PER_SECOND, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in minutes. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in minutes, truncated
/// @example (:wat::time::minutes (:wat::time::Minute 5)) #=> 5
/// @see     :wat::time::Minute
#[wat_intrinsic(":wat::time::minutes")]
pub(crate) fn eval_time_minutes(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::minutes", NANOS_PER_MINUTE, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in hours. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in hours, truncated
/// @example (:wat::time::hours (:wat::time::Hour 5)) #=> 5
/// @see     :wat::time::Hour
#[wat_intrinsic(":wat::time::hours")]
pub(crate) fn eval_time_hours(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::hours", NANOS_PER_HOUR, d, list_span, env, sym).map_err(Into::into)
}

/// Reads a Duration's length in days. Truncating.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     d :wat::time::Duration the duration to read
/// @ret     :wat::core::i64 the duration's length in days, truncated
/// @example (:wat::time::days (:wat::time::Day 5)) #=> 5
/// @see     :wat::time::Day
#[wat_intrinsic(":wat::time::days")]
pub(crate) fn eval_time_days(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_readout(":wat::time::days", NANOS_PER_DAY, d, list_span, env, sym).map_err(Into::into)
}

// ─── Arc 097 — Polymorphic Instant ± Duration arithmetic ────────────
//
// `:wat::time::-` dispatches on the RHS Value variant:
//   Instant - Duration -> Instant   (subtract interval)
//   Instant - Instant  -> Duration  (elapsed between, panics if negative)
//
// `:wat::time::+` is single-arm:
//   Instant + Duration -> Instant   (advance by interval)
//
// Same surface as ActiveSupport's `time1 - time2 = duration` and
// `time - 1.hour = time`. The runtime checks the RHS variant and
// picks the right behavior at call time. The type checker
// (check.rs::infer_polymorphic_time_arith) does the same dispatch
// at expansion time and reports the result type — it special-cases
// `:wat::time::+`/`-` directly (never calls `env.register` for them),
// so neither carries a fixed TypeScheme for the doc-vs-checker test to
// compare against.
//
// Per arc 097 §2: Durations are non-negative. If `(- a b)` would
// produce a negative interval (a is before b), panic with a
// diagnostic asking the user to subtract in the other order.

/// Instant ± arithmetic, polymorphic on the RHS variant: `Instant - Duration
/// -> Instant` (subtract interval); `Instant - Instant -> Duration` (elapsed,
/// panics if negative — Durations are non-negative; subtract in the other
/// order).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Arithmetic
/// @arg     a :wat::time::Instant the instant to subtract from
/// @arg     b :wat::time::Instant the RHS — a Duration or an Instant, dispatched at runtime
/// @ret     :wat::time::Instant Instant when RHS is a Duration; :wat::time::Duration (elapsed, non-negative) when RHS is an Instant
/// @example (:wat::time::- (:wat::time::at 10) (:wat::time::at 4)) #=> (:wat::time::Second 6)
/// @see     :wat::time::+
#[wat_intrinsic(":wat::time::-")]
pub(crate) fn eval_time_sub(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::-";
    let a = eval(a, env, sym)?;
    let b = eval(b, env, sym)?.value_owned();
    let a_inst = require_instant(OP, a, list_span)?;
    match b {
        Value::Duration(ns) => {
            // Instant - Duration -> Instant.
            // ns is non-negative (constructor invariant); subtract
            // by adding chrono::Duration::nanoseconds(-ns).
            let new_inst = a_inst
                .checked_sub_signed(chrono::Duration::nanoseconds(ns))
                // chrono range error — evaluated Values have no AST trace; list_span is the best available location
                .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "result-Instant in chrono representable range",
                    got: Box::new(ValueSnapshot::unavailable("out-of-range subtraction"))
                }))?;
            Ok(Value::Instant(new_inst))
        }
        Value::Instant(b_inst) => {
            // Instant - Instant -> Duration. Compute elapsed via
            // chrono's signed_duration_since; panic if negative
            // per §2.
            let dur = a_inst.signed_duration_since(b_inst);
            let ns = dur.num_nanoseconds().ok_or_else(|| {
                // chrono range error — evaluated Values have no AST trace; list_span is the best available location
                RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "elapsed nanoseconds in i64 range",
                    got: Box::new(ValueSnapshot::unavailable("out-of-range duration"))
                })
            })?;
            if ns < 0 {
                panic!(
                    "({} a b): would produce negative Duration ({} ns); \
                     Durations are non-negative — subtract in the other \
                     order ((:wat::time::- b a)) or use the chronological \
                     direction your script actually means",
                    OP, ns
                );
            }
            Ok(Value::Duration(ns))
        }
        // b is an evaluated Value with no AST trace at match point; list_span is the best available location
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::time::Duration or wat::time::Instant",
            got: Box::new(ValueSnapshot::of(&other))
        }).into()),
    }
}

/// Advances an Instant by a Duration.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Arithmetic
/// @arg     a :wat::time::Instant the instant to advance
/// @arg     b :wat::time::Duration the interval to advance by
/// @ret     :wat::time::Instant the instant advanced by the duration
/// @example (:wat::time::+ (:wat::time::at 4) (:wat::time::Second 6)) #=> (:wat::time::at 10)
/// @see     :wat::time::-
#[wat_intrinsic(":wat::time::+")]
pub(crate) fn eval_time_add(
    a: &WatAST,
    b: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::+";
    let a = eval(a, env, sym)?;
    let b = eval(b, env, sym)?.value_owned();
    let a_inst = require_instant(OP, a, list_span)?;
    let ns = match b {
        Value::Duration(ns) => ns,
        other => {
            // b is an evaluated Value with no AST trace at match point; list_span is the best available location
            return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::time::Duration",
                got: Box::new(ValueSnapshot::of(&other))
            }).into())
        }
    };
    let new_inst = a_inst
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range addition"))
        }))?;
    Ok(Value::Instant(new_inst))
}

// ─── Arc 097 slice 3 — `ago` / `from-now` composers ─────────────────
//
// ActiveSupport-flavored "X ago" / "X from now" — relative to (now).
// Each composer takes a Duration; computes Instant relative to wall-
// clock now. Same semantic as Ruby's `1.hour.ago` and `2.days.from_now`.
// Both read the wall clock (`Utc::now()`) — Nondeterministic.

/// Instant `duration` before now. Equivalent to `(:wat::time::-
/// (:wat::time::now) duration)`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     d :wat::time::Duration the interval before now
/// @ret     :wat::time::Instant the instant `d` before wall-clock now
/// @example-norun (:wat::time::ago (:wat::time::Hour 1)) #=> #inst "one hour before now"
/// @see     :wat::time::from-now
#[wat_intrinsic(":wat::time::ago")]
pub(crate) fn eval_time_ago(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::ago";
    let ns = require_duration(OP, eval(d, env, sym)?, list_span)?;
    let now = Utc::now();
    let result = now
        .checked_sub_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range subtraction"))
        }))?;
    Ok(Value::Instant(result))
}

/// Instant `duration` after now. Equivalent to `(:wat::time::+
/// (:wat::time::now) duration)`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     d :wat::time::Duration the interval after now
/// @ret     :wat::time::Instant the instant `d` after wall-clock now
/// @example-norun (:wat::time::from-now (:wat::time::Hour 1)) #=> #inst "one hour after now"
/// @see     :wat::time::ago
#[wat_intrinsic(":wat::time::from-now")]
pub(crate) fn eval_time_from_now(
    d: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::time::from-now";
    let ns = require_duration(OP, eval(d, env, sym)?, list_span)?;
    let now = Utc::now();
    let result = now
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range addition"))
        }))?;
    Ok(Value::Instant(result))
}

// ─── Arc 097 slice 4 — pre-composed unit-ago / unit-from-now ────────
//
// 14 sugars (7 units × {ago, from-now}). Each computes the relative
// Instant in one call: `(hours-ago 1)` instead of
// `(:wat::time::ago (:wat::time::Hour 1))`. Reads cleaner at every
// callsite. Each reads the wall clock (`Utc::now()`) via the shared
// `unit_ago`/`unit_from_now` helper — Nondeterministic.

fn unit_ago(
    op: &'static str,
    unit_nanos: i64,
    n: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let n = require_i64(op, eval(n, env, sym)?, list_span)?;
    if n < 0 {
        panic!(
            "({} {}): count must be non-negative; \
             X-ago / X-from-now express past / future intervals — \
             direction is in the verb, not the count",
            op, n
        );
    }
    let nanos = n.checked_mul(unit_nanos).unwrap_or_else(|| {
        panic!(
            "({} {}): overflows representable Duration; \
             max for this unit is ~{}",
            op,
            n,
            i64::MAX / unit_nanos
        )
    });
    let result = Utc::now()
        .checked_sub_signed(chrono::Duration::nanoseconds(nanos))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range subtraction"))
        }))?;
    Ok(Value::Instant(result))
}

fn unit_from_now(
    op: &'static str,
    unit_nanos: i64,
    n: &WatAST,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let n = require_i64(op, eval(n, env, sym)?, list_span)?;
    if n < 0 {
        panic!(
            "({} {}): count must be non-negative; \
             X-ago / X-from-now express past / future intervals — \
             direction is in the verb, not the count",
            op, n
        );
    }
    let nanos = n.checked_mul(unit_nanos).unwrap_or_else(|| {
        panic!(
            "({} {}): overflows representable Duration; \
             max for this unit is ~{}",
            op,
            n,
            i64::MAX / unit_nanos
        )
    });
    let result = Utc::now()
        .checked_add_signed(chrono::Duration::nanoseconds(nanos))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(ValueSnapshot::unavailable("out-of-range addition"))
        }))?;
    Ok(Value::Instant(result))
}

// ─── Per-unit ago helpers ───────────────────────────────────────────

/// N nanoseconds before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of nanoseconds before now (non-negative)
/// @ret     :wat::time::Instant the instant N nanoseconds before wall-clock now
/// @example-norun (:wat::time::nanoseconds-ago 5) #=> #inst "5ns before now"
/// @see     :wat::time::nanoseconds-from-now
#[wat_intrinsic(":wat::time::nanoseconds-ago")]
pub(crate) fn eval_time_nanoseconds_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::nanoseconds-ago", 1, n, list_span, env, sym).map_err(Into::into)
}

/// N microseconds before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of microseconds before now (non-negative)
/// @ret     :wat::time::Instant the instant N microseconds before wall-clock now
/// @example-norun (:wat::time::microseconds-ago 5) #=> #inst "5us before now"
/// @see     :wat::time::microseconds-from-now
#[wat_intrinsic(":wat::time::microseconds-ago")]
pub(crate) fn eval_time_microseconds_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::microseconds-ago", NANOS_PER_MICRO, n, list_span, env, sym).map_err(Into::into)
}

/// N milliseconds before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of milliseconds before now (non-negative)
/// @ret     :wat::time::Instant the instant N milliseconds before wall-clock now
/// @example-norun (:wat::time::milliseconds-ago 5) #=> #inst "5ms before now"
/// @see     :wat::time::milliseconds-from-now
#[wat_intrinsic(":wat::time::milliseconds-ago")]
pub(crate) fn eval_time_milliseconds_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::milliseconds-ago", NANOS_PER_MILLI, n, list_span, env, sym).map_err(Into::into)
}

/// N seconds before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of seconds before now (non-negative)
/// @ret     :wat::time::Instant the instant N seconds before wall-clock now
/// @example-norun (:wat::time::seconds-ago 5) #=> #inst "5s before now"
/// @see     :wat::time::seconds-from-now
#[wat_intrinsic(":wat::time::seconds-ago")]
pub(crate) fn eval_time_seconds_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::seconds-ago", NANOS_PER_SECOND, n, list_span, env, sym).map_err(Into::into)
}

/// N minutes before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of minutes before now (non-negative)
/// @ret     :wat::time::Instant the instant N minutes before wall-clock now
/// @example-norun (:wat::time::minutes-ago 5) #=> #inst "5m before now"
/// @see     :wat::time::minutes-from-now
#[wat_intrinsic(":wat::time::minutes-ago")]
pub(crate) fn eval_time_minutes_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::minutes-ago", NANOS_PER_MINUTE, n, list_span, env, sym).map_err(Into::into)
}

/// N hours before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of hours before now (non-negative)
/// @ret     :wat::time::Instant the instant N hours before wall-clock now
/// @example-norun (:wat::time::hours-ago 5) #=> #inst "5h before now"
/// @see     :wat::time::hours-from-now
#[wat_intrinsic(":wat::time::hours-ago")]
pub(crate) fn eval_time_hours_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::hours-ago", NANOS_PER_HOUR, n, list_span, env, sym).map_err(Into::into)
}

/// N days before now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of days before now (non-negative)
/// @ret     :wat::time::Instant the instant N days before wall-clock now
/// @example-norun (:wat::time::days-ago 5) #=> #inst "5d before now"
/// @see     :wat::time::days-from-now
#[wat_intrinsic(":wat::time::days-ago")]
pub(crate) fn eval_time_days_ago(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_ago(":wat::time::days-ago", NANOS_PER_DAY, n, list_span, env, sym).map_err(Into::into)
}

// ─── Per-unit from-now helpers ──────────────────────────────────────

/// N nanoseconds after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of nanoseconds after now (non-negative)
/// @ret     :wat::time::Instant the instant N nanoseconds after wall-clock now
/// @example-norun (:wat::time::nanoseconds-from-now 5) #=> #inst "5ns after now"
/// @see     :wat::time::nanoseconds-ago
#[wat_intrinsic(":wat::time::nanoseconds-from-now")]
pub(crate) fn eval_time_nanoseconds_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::nanoseconds-from-now", 1, n, list_span, env, sym).map_err(Into::into)
}

/// N microseconds after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of microseconds after now (non-negative)
/// @ret     :wat::time::Instant the instant N microseconds after wall-clock now
/// @example-norun (:wat::time::microseconds-from-now 5) #=> #inst "5us after now"
/// @see     :wat::time::microseconds-ago
#[wat_intrinsic(":wat::time::microseconds-from-now")]
pub(crate) fn eval_time_microseconds_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::microseconds-from-now", NANOS_PER_MICRO, n, list_span, env, sym)
        .map_err(Into::into)
}

/// N milliseconds after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of milliseconds after now (non-negative)
/// @ret     :wat::time::Instant the instant N milliseconds after wall-clock now
/// @example-norun (:wat::time::milliseconds-from-now 5) #=> #inst "5ms after now"
/// @see     :wat::time::milliseconds-ago
#[wat_intrinsic(":wat::time::milliseconds-from-now")]
pub(crate) fn eval_time_milliseconds_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::milliseconds-from-now", NANOS_PER_MILLI, n, list_span, env, sym)
        .map_err(Into::into)
}

/// N seconds after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of seconds after now (non-negative)
/// @ret     :wat::time::Instant the instant N seconds after wall-clock now
/// @example-norun (:wat::time::seconds-from-now 5) #=> #inst "5s after now"
/// @see     :wat::time::seconds-ago
#[wat_intrinsic(":wat::time::seconds-from-now")]
pub(crate) fn eval_time_seconds_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::seconds-from-now", NANOS_PER_SECOND, n, list_span, env, sym)
        .map_err(Into::into)
}

/// N minutes after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of minutes after now (non-negative)
/// @ret     :wat::time::Instant the instant N minutes after wall-clock now
/// @example-norun (:wat::time::minutes-from-now 5) #=> #inst "5m after now"
/// @see     :wat::time::minutes-ago
#[wat_intrinsic(":wat::time::minutes-from-now")]
pub(crate) fn eval_time_minutes_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::minutes-from-now", NANOS_PER_MINUTE, n, list_span, env, sym)
        .map_err(Into::into)
}

/// N hours after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of hours after now (non-negative)
/// @ret     :wat::time::Instant the instant N hours after wall-clock now
/// @example-norun (:wat::time::hours-from-now 5) #=> #inst "5h after now"
/// @see     :wat::time::hours-ago
#[wat_intrinsic(":wat::time::hours-from-now")]
pub(crate) fn eval_time_hours_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::hours-from-now", NANOS_PER_HOUR, n, list_span, env, sym).map_err(Into::into)
}

/// N days after now.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Nondeterministic
/// @Category      Entropic
/// @arg     n :wat::core::i64 the count of days after now (non-negative)
/// @ret     :wat::time::Instant the instant N days after wall-clock now
/// @example-norun (:wat::time::days-from-now 5) #=> #inst "5d after now"
/// @see     :wat::time::days-ago
#[wat_intrinsic(":wat::time::days-from-now")]
pub(crate) fn eval_time_days_from_now(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    unit_from_now(":wat::time::days-from-now", NANOS_PER_DAY, n, list_span, env, sym).map_err(Into::into)
}

// ─── Helpers — local to this module ─────────────────────────────────

fn require_i64(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<i64, RuntimeError> {
    match tv.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "i64",
            got: Box::new(ValueSnapshot::of(&other))
        })),
    }
}

fn require_string(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<String, RuntimeError> {
    match tv.value_owned() {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: Box::new(ValueSnapshot::of(&other))
        })),
    }
}

fn require_instant(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<DateTime<Utc>, RuntimeError> {
    match tv.value_owned() {
        Value::Instant(dt) => Ok(dt),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Instant",
            got: Box::new(ValueSnapshot::of(&other))
        })),
    }
}

fn require_duration(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<i64, RuntimeError> {
    match tv.value_owned() {
        Value::Duration(ns) => Ok(ns),
        other => Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Duration",
            got: Box::new(ValueSnapshot::of(&other))
        })),
    }
}
