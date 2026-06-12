//! `:wat::time::Instant` — a single wall-clock value type. Arc 056.
//!
//! **Lineage: Java / Clojure.** Single `Instant` covers both
//! "when did this happen?" and "how long did this take?" — the latter
//! is `(now)` before, `(now)` after, subtract integer accessors. No
//! separate monotonic / `Duration` type. Rust's `SystemTime` /
//! `Instant` split is the outlier; this module follows the broader
//! lineage (Java `java.time.Instant`, Clojure
//! `(System/currentTimeMillis)`, JS `Date`, Python `datetime`,
//! SQL `TIMESTAMP`).
//!
//! UTC only. ISO 8601 / RFC 3339 round-trips. Sub-second precision
//! up to nanoseconds. i64 nanos saturates at year ~2262.
//!
//! Backing: `chrono::DateTime<chrono::Utc>`. `Value::Instant`
//! variant in [`crate::runtime::Value`]. The dispatch arms in
//! `runtime.rs` invoke the `eval_time_*` functions defined here;
//! the type schemes in `check.rs` register the surface.
//!
//! Surface (9 primitives at `:wat::time::*`):
//!
//! ```text
//! :wat::time::now              -> :wat::time::Instant
//! :wat::time::at         (i64) -> :wat::time::Instant
//! :wat::time::at-millis  (i64) -> :wat::time::Instant
//! :wat::time::at-nanos   (i64) -> :wat::time::Instant
//! :wat::time::from-iso8601 (String) -> :Option<wat::time::Instant>
//! :wat::time::to-iso8601 (Instant, i64) -> :String
//! :wat::time::epoch-seconds    (Instant)   -> :i64
//! :wat::time::epoch-millis     (Instant)   -> :i64
//! :wat::time::epoch-nanos      (Instant)   -> :i64
//! :wat::time::nanoseconds      (Duration)  -> :i64
//! :wat::time::microseconds     (Duration)  -> :i64
//! :wat::time::milliseconds     (Duration)  -> :i64
//! :wat::time::seconds          (Duration)  -> :i64
//! :wat::time::minutes          (Duration)  -> :i64
//! :wat::time::hours            (Duration)  -> :i64
//! :wat::time::days             (Duration)  -> :i64
//! ```
//!
//! ## Namespace placement (Q10 — `:wat::time::*`, not `:wat::std::*`)
//!
//! `:wat::std::*` is the *pure* stdlib — referentially-transparent
//! algorithms and data utilities. `:wat::io::*` is world-interaction:
//! its returns depend on world state. `(:wat::time::now)` observes
//! the system clock — same category as `:wat::io::*`. Time lives
//! at the same nesting depth as `:wat::io::*`, not nested under
//! `:wat::std::*`.

use chrono::{DateTime, SecondsFormat, TimeZone, Utc};

use std::sync::{Arc, Mutex};

// ─── Process-level boot clock ─────────────────────────────────────────
//
// pid-keyed and fork-safe: captures `now` lazily on first call; re-captures
// across a fork (stored pid != current pid) so a forked `:process` peer
// measures its OWN boot, not the parent's inherited value.
//
// Both fns are `pub` so the test crate can reach them as `wat::time::*`.

static PROCESS_BOOT: Mutex<Option<(u32, DateTime<Utc>)>> = Mutex::new(None);

/// This process's boot instant. Captured lazily on first call; re-captured
/// across a fork (pid change) so a `:process` peer measures its own boot.
/// pid-keyed.
pub fn process_boot_instant() -> DateTime<Utc> {
    let pid = std::process::id();
    let mut g = PROCESS_BOOT.lock().unwrap_or_else(|e| e.into_inner());
    match *g {
        Some((p, inst)) if p == pid => inst,
        _ => {
            let now = Utc::now();
            *g = Some((pid, now));
            now
        }
    }
}

/// Explicitly set this process's boot instant (wat-cli at its earliest point;
/// tests inject a known value for deterministic timing). pid-keyed to the caller.
pub fn set_process_boot_instant(inst: DateTime<Utc>) {
    let pid = std::process::id();
    *PROCESS_BOOT.lock().unwrap_or_else(|e| e.into_inner()) = Some((pid, inst));
}

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};
use crate::value::TrackedValue;
use crate::span::Span;

// ─── Constructors ────────────────────────────────────────────────────

/// `(:wat::time::now) -> :wat::time::Instant` — current wall-clock time.
pub(crate) fn eval_time_now(args: &[WatAST], list_span: &Span) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::now";
    if !args.is_empty() {
        return Err(RuntimeError { span: args[0].span().clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len()
        } });
    }
    Ok(Value::Instant(Utc::now()))
}

/// `(:wat::time::at epoch-seconds:i64) -> :wat::time::Instant`. From
/// integer seconds since 1970-01-01T00:00:00Z. Negative values are
/// pre-epoch and behave per chrono.
pub(crate) fn eval_time_at(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let secs = require_i64(OP, eval(&args[0], env, sym)?, list_span)?;
    let dt = Utc.timestamp_opt(secs, 0).single().ok_or_else(|| {
        // chrono range error — secs is a plain i64 from an evaluated Value; list_span is the best available location
        RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "epoch-seconds in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range i64"))
        } }
    })?;
    Ok(Value::Instant(dt))
}

/// `(:wat::time::at-millis epoch-ms:i64) -> :wat::time::Instant`.
pub(crate) fn eval_time_at_millis(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at-millis";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ms = require_i64(OP, eval(&args[0], env, sym)?, list_span)?;
    let dt = Utc.timestamp_millis_opt(ms).single().ok_or_else(|| {
        // chrono range error — ms is a plain i64 from an evaluated Value; list_span is the best available location
        RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "epoch-ms in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range i64"))
        } }
    })?;
    Ok(Value::Instant(dt))
}

/// `(:wat::time::at-nanos epoch-ns:i64) -> :wat::time::Instant`.
/// i64 ns saturates at year ~2262.
pub(crate) fn eval_time_at_nanos(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at-nanos";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ns = require_i64(OP, eval(&args[0], env, sym)?, list_span)?;
    Ok(Value::Instant(Utc.timestamp_nanos(ns)))
}

/// `(:wat::time::from-iso8601 s:String) -> :Option<wat::time::Instant>`.
/// `:None` on parse failure. Accepts `parse_from_rfc3339` grammar
/// (the practical ISO 8601 subset).
pub(crate) fn eval_time_from_iso8601(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::from-iso8601";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let s = require_string(OP, eval(&args[0], env, sym)?, list_span)?;
    let parsed = DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc));
    let inner = parsed.map(Value::Instant);
    Ok(Value::Option(Arc::new(inner)))
}

// ─── Formatter ───────────────────────────────────────────────────────

/// `(:wat::time::to-iso8601 i:Instant digits:i64) -> :String`. ISO
/// 8601 / RFC 3339 with N fractional second digits. `digits` is
/// clamped to `[0, 9]`; output always UTC (`Z` suffix).
pub(crate) fn eval_time_to_iso8601(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::to-iso8601";
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?, list_span)?;
    let digits_raw = require_i64(OP, eval(&args[1], env, sym)?, list_span)?;
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

/// `(:wat::time::epoch-seconds i:Instant) -> :i64`. Truncating;
/// sub-second precision lost.
pub(crate) fn eval_time_epoch_seconds(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-seconds";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?, list_span)?;
    Ok(Value::i64(inst.timestamp()))
}

/// `(:wat::time::epoch-millis i:Instant) -> :i64`. Truncating to ms.
pub(crate) fn eval_time_epoch_millis(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-millis";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?, list_span)?;
    Ok(Value::i64(inst.timestamp_millis()))
}

/// `(:wat::time::epoch-nanos i:Instant) -> :i64`. Panics if the
/// instant is outside i64-nanosecond representable range
/// (i.e., before ~1677 or after ~2262).
pub(crate) fn eval_time_epoch_nanos(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-nanos";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?, list_span)?;
    let ns = inst.timestamp_nanos_opt().ok_or_else(|| {
        // chrono range error — inst is a plain DateTime from an evaluated Value; list_span is the best available location
        RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "instant in i64-nanosecond range (~1677 to ~2262)",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range instant"))
        } }
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
// The shared `unit_constructor` helper does arity check, type check,
// negativity check, overflow-on-multiply check; the seven public
// functions just thread their unit's nanos-per-unit constant.

const NANOS_PER_MICRO: i64 = 1_000;
const NANOS_PER_MILLI: i64 = 1_000_000;
const NANOS_PER_SECOND: i64 = 1_000_000_000;
const NANOS_PER_MINUTE: i64 = 60 * NANOS_PER_SECOND;
const NANOS_PER_HOUR: i64 = 60 * NANOS_PER_MINUTE;
const NANOS_PER_DAY: i64 = 24 * NANOS_PER_HOUR;

fn unit_constructor(
    op: &'static str,
    unit_nanos: i64,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?, list_span)?;
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

/// `(:wat::time::Nanosecond n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_nanosecond(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Nanosecond", 1, args, list_span, env, sym)
}

/// `(:wat::time::Microsecond n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_microsecond(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Microsecond", NANOS_PER_MICRO, args, list_span, env, sym)
}

/// `(:wat::time::Millisecond n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_millisecond(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Millisecond", NANOS_PER_MILLI, args, list_span, env, sym)
}

/// `(:wat::time::Second n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_second(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Second", NANOS_PER_SECOND, args, list_span, env, sym)
}

/// `(:wat::time::Minute n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_minute(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Minute", NANOS_PER_MINUTE, args, list_span, env, sym)
}

/// `(:wat::time::Hour n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_hour(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Hour", NANOS_PER_HOUR, args, list_span, env, sym)
}

/// `(:wat::time::Day n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_day(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Day", NANOS_PER_DAY, args, list_span, env, sym)
}

// ─── Duration readout family — symmetric OUT half of the constructors ──
//
// Seven `:wat::time::<unit>` verbs (bare unit-plural) mirror the seven
// constructors: capitalized `Second` constructs a Duration, lowercase-plural
// `seconds` reads one out. The unit word IS the accessor — `(seconds d)` says
// exactly what it does, cohering with the `seconds-ago` / `seconds-from-now`
// families (same unit words, same spelling level).
// Each takes a `:wat::time::Duration` and returns `:i64` by dividing
// the stored nanosecond count by the unit's nanos-per-unit constant,
// truncating toward zero (same behaviour as `epoch-millis`).
//
// The shared `unit_readout` helper does arity check, type check, and
// division; the seven public functions just thread their constant.

fn unit_readout(
    op: &'static str,
    unit_nanos: i64,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ns = require_duration(op, eval(&args[0], env, sym)?, list_span)?;
    Ok(Value::i64(ns / unit_nanos))
}

/// `(:wat::time::nanoseconds d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_nanoseconds(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::nanoseconds", 1, args, list_span, env, sym)
}

/// `(:wat::time::microseconds d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_microseconds(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::microseconds", NANOS_PER_MICRO, args, list_span, env, sym)
}

/// `(:wat::time::milliseconds d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_milliseconds(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::milliseconds", NANOS_PER_MILLI, args, list_span, env, sym)
}

/// `(:wat::time::seconds d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_seconds(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::seconds", NANOS_PER_SECOND, args, list_span, env, sym)
}

/// `(:wat::time::minutes d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_minutes(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::minutes", NANOS_PER_MINUTE, args, list_span, env, sym)
}

/// `(:wat::time::hours d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_hours(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::hours", NANOS_PER_HOUR, args, list_span, env, sym)
}

/// `(:wat::time::days d:Duration) -> :i64`. Truncating.
pub(crate) fn eval_time_days(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_readout(":wat::time::days", NANOS_PER_DAY, args, list_span, env, sym)
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
// at expansion time and reports the result type.
//
// Per arc 097 §2: Durations are non-negative. If `(- a b)` would
// produce a negative interval (a is before b), panic with a
// diagnostic asking the user to subtract in the other order.
//
// Duration ± Duration is NOT in this slice — defer until a real
// consumer demands it. Users can compose by constructing the
// duration they want directly (`(Hour 1)`, `(Minute 30)`).

/// `(:wat::time::- a b)` — polymorphic on RHS variant.
pub(crate) fn eval_time_sub(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::-";
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?.value_owned();
    let a_inst = require_instant(OP, a, list_span)?;
    match b {
        Value::Duration(ns) => {
            // Instant - Duration -> Instant.
            // ns is non-negative (constructor invariant); subtract
            // by adding chrono::Duration::nanoseconds(-ns).
            let new_inst = a_inst
                .checked_sub_signed(chrono::Duration::nanoseconds(ns))
                // chrono range error — evaluated Values have no AST trace; list_span is the best available location
                .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "result-Instant in chrono representable range",
                    got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range subtraction"))
                } })?;
            Ok(Value::Instant(new_inst))
        }
        Value::Instant(b_inst) => {
            // Instant - Instant -> Duration. Compute elapsed via
            // chrono's signed_duration_since; panic if negative
            // per §2.
            let dur = a_inst.signed_duration_since(b_inst);
            let ns = dur.num_nanoseconds().ok_or_else(|| {
                // chrono range error — evaluated Values have no AST trace; list_span is the best available location
                RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "elapsed nanoseconds in i64 range",
                    got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range duration"))
                } }
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
        other => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "wat::time::Duration or wat::time::Instant",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

/// `(:wat::time::+ instant duration) -> Instant`.
pub(crate) fn eval_time_add(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::+";
    if args.len() != 2 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len()
        } });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?.value_owned();
    let a_inst = require_instant(OP, a, list_span)?;
    let ns = match b {
        Value::Duration(ns) => ns,
        other => {
            // b is an evaluated Value with no AST trace at match point; list_span is the best available location
            return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "wat::time::Duration",
                got: Box::new(crate::runtime::ValueSnapshot::of(&other))
            } })
        }
    };
    let new_inst = a_inst
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range addition"))
        } })?;
    Ok(Value::Instant(new_inst))
}

// ─── Arc 097 slice 3 — `ago` / `from-now` composers ─────────────────
//
// ActiveSupport-flavored "X ago" / "X from now" — relative to (now).
// Each composer takes a Duration; computes Instant relative to wall-
// clock now. Same semantic as Ruby's `1.hour.ago` and `2.days.from_now`.

/// `(:wat::time::ago duration) -> :wat::time::Instant`. Equivalent to
/// `(:wat::time::- (:wat::time::now) duration)`.
pub(crate) fn eval_time_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::ago";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ns = require_duration(OP, eval(&args[0], env, sym)?, list_span)?;
    let now = Utc::now();
    let result = now
        .checked_sub_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range subtraction"))
        } })?;
    Ok(Value::Instant(result))
}

/// `(:wat::time::from-now duration) -> :wat::time::Instant`. Equivalent
/// to `(:wat::time::+ (:wat::time::now) duration)`.
pub(crate) fn eval_time_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::from-now";
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let ns = require_duration(OP, eval(&args[0], env, sym)?, list_span)?;
    let now = Utc::now();
    let result = now
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        // chrono range error — evaluated Values have no AST trace; list_span is the best available location
        .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range addition"))
        } })?;
    Ok(Value::Instant(result))
}

// ─── Arc 097 slice 4 — pre-composed unit-ago / unit-from-now ────────
//
// 14 sugars (7 units × {ago, from-now}). Each computes the relative
// Instant in one call: `(hours-ago 1)` instead of
// `(:wat::time::ago (:wat::time::Hour 1))`. Reads cleaner at every
// callsite.
//
// Implementation: each takes :i64, applies the unit's nanos
// multiplier through the same construction guards as slice 1
// (negative input → panic; overflow → panic), then computes the
// relative Instant via slice 3's add/sub against `now`.

fn unit_ago(
    op: &'static str,
    unit_nanos: i64,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?, list_span)?;
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
        .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range subtraction"))
        } })?;
    Ok(Value::Instant(result))
}

fn unit_from_now(
    op: &'static str,
    unit_nanos: i64,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len()
        } });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?, list_span)?;
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
        .ok_or_else(|| RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: Box::new(crate::runtime::ValueSnapshot::unavailable("out-of-range addition"))
        } })?;
    Ok(Value::Instant(result))
}

// ─── Per-unit ago helpers ───────────────────────────────────────────

pub(crate) fn eval_time_nanoseconds_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::nanoseconds-ago", 1, args, list_span, env, sym)
}

pub(crate) fn eval_time_microseconds_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::microseconds-ago",
        NANOS_PER_MICRO,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_milliseconds_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::milliseconds-ago",
        NANOS_PER_MILLI,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_seconds_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::seconds-ago",
        NANOS_PER_SECOND,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_minutes_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::minutes-ago",
        NANOS_PER_MINUTE,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_hours_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::hours-ago", NANOS_PER_HOUR, args, list_span, env, sym)
}

pub(crate) fn eval_time_days_ago(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::days-ago", NANOS_PER_DAY, args, list_span, env, sym)
}

// ─── Per-unit from-now helpers ──────────────────────────────────────

pub(crate) fn eval_time_nanoseconds_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(":wat::time::nanoseconds-from-now", 1, args, list_span, env, sym)
}

pub(crate) fn eval_time_microseconds_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::microseconds-from-now",
        NANOS_PER_MICRO,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_milliseconds_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::milliseconds-from-now",
        NANOS_PER_MILLI,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_seconds_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::seconds-from-now",
        NANOS_PER_SECOND,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_minutes_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::minutes-from-now",
        NANOS_PER_MINUTE,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_hours_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::hours-from-now",
        NANOS_PER_HOUR,
        args,
        list_span,
        env,
        sym,
    )
}

pub(crate) fn eval_time_days_from_now(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(":wat::time::days-from-now", NANOS_PER_DAY, args, list_span, env, sym)
}

// ─── Helpers — local to this module ─────────────────────────────────

fn require_i64(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<i64, RuntimeError> {
    match tv.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "i64",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

fn require_string(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<String, RuntimeError> {
    match tv.value_owned() {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

fn require_instant(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<DateTime<Utc>, RuntimeError> {
    match tv.value_owned() {
        Value::Instant(dt) => Ok(dt),
        other => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Instant",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}

fn require_duration(op: &'static str, tv: TrackedValue, list_span: &Span) -> Result<i64, RuntimeError> {
    match tv.value_owned() {
        Value::Duration(ns) => Ok(ns),
        other => Err(RuntimeError { span: list_span.clone(), kind: RuntimeErrorKind::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Duration",
            got: Box::new(crate::runtime::ValueSnapshot::of(&other))
        } }),
    }
}
