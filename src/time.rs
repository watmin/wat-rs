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
//! :wat::time::epoch-seconds (Instant) -> :i64
//! :wat::time::epoch-millis  (Instant) -> :i64
//! :wat::time::epoch-nanos   (Instant) -> :i64
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

use std::sync::Arc;

use crate::ast::WatAST;
use crate::runtime::{eval, Environment, RuntimeError, SymbolTable, Value};

// ─── Constructors ────────────────────────────────────────────────────

/// `(:wat::time::now) -> :wat::time::Instant` — current wall-clock time.
pub(crate) fn eval_time_now(args: &[WatAST]) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::now";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
        });
    }
    Ok(Value::Instant(Utc::now()))
}

/// `(:wat::time::at epoch-seconds:i64) -> :wat::time::Instant`. From
/// integer seconds since 1970-01-01T00:00:00Z. Negative values are
/// pre-epoch and behave per chrono.
pub(crate) fn eval_time_at(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let secs = require_i64(OP, eval(&args[0], env, sym)?)?;
    let dt = Utc.timestamp_opt(secs, 0).single().ok_or_else(|| {
        RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "epoch-seconds in chrono representable range",
            got: "out-of-range i64",
        }
    })?;
    Ok(Value::Instant(dt))
}

/// `(:wat::time::at-millis epoch-ms:i64) -> :wat::time::Instant`.
pub(crate) fn eval_time_at_millis(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at-millis";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let ms = require_i64(OP, eval(&args[0], env, sym)?)?;
    let dt = Utc.timestamp_millis_opt(ms).single().ok_or_else(|| {
        RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "epoch-ms in chrono representable range",
            got: "out-of-range i64",
        }
    })?;
    Ok(Value::Instant(dt))
}

/// `(:wat::time::at-nanos epoch-ns:i64) -> :wat::time::Instant`.
/// i64 ns saturates at year ~2262.
pub(crate) fn eval_time_at_nanos(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::at-nanos";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let ns = require_i64(OP, eval(&args[0], env, sym)?)?;
    Ok(Value::Instant(Utc.timestamp_nanos(ns)))
}

/// `(:wat::time::from-iso8601 s:String) -> :Option<wat::time::Instant>`.
/// `:None` on parse failure. Accepts `parse_from_rfc3339` grammar
/// (the practical ISO 8601 subset).
pub(crate) fn eval_time_from_iso8601(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::from-iso8601";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let s = require_string(OP, eval(&args[0], env, sym)?)?;
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::to-iso8601";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?)?;
    let digits_raw = require_i64(OP, eval(&args[1], env, sym)?)?;
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-seconds";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?)?;
    Ok(Value::i64(inst.timestamp()))
}

/// `(:wat::time::epoch-millis i:Instant) -> :i64`. Truncating to ms.
pub(crate) fn eval_time_epoch_millis(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-millis";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?)?;
    Ok(Value::i64(inst.timestamp_millis()))
}

/// `(:wat::time::epoch-nanos i:Instant) -> :i64`. Panics if the
/// instant is outside i64-nanosecond representable range
/// (i.e., before ~1677 or after ~2262).
pub(crate) fn eval_time_epoch_nanos(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::epoch-nanos";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let inst = require_instant(OP, eval(&args[0], env, sym)?)?;
    let ns = inst.timestamp_nanos_opt().ok_or_else(|| {
        RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "instant in i64-nanosecond range (~1677 to ~2262)",
            got: "out-of-range instant",
        }
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?)?;
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Nanosecond", 1, args, env, sym)
}

/// `(:wat::time::Microsecond n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_microsecond(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Microsecond", NANOS_PER_MICRO, args, env, sym)
}

/// `(:wat::time::Millisecond n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_millisecond(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Millisecond", NANOS_PER_MILLI, args, env, sym)
}

/// `(:wat::time::Second n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_second(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Second", NANOS_PER_SECOND, args, env, sym)
}

/// `(:wat::time::Minute n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_minute(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Minute", NANOS_PER_MINUTE, args, env, sym)
}

/// `(:wat::time::Hour n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_hour(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Hour", NANOS_PER_HOUR, args, env, sym)
}

/// `(:wat::time::Day n:i64) -> :wat::time::Duration`.
pub(crate) fn eval_time_unit_day(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_constructor(":wat::time::Day", NANOS_PER_DAY, args, env, sym)
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::-";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?;
    let a_inst = require_instant(OP, a)?;
    match b {
        Value::Duration(ns) => {
            // Instant - Duration -> Instant.
            // ns is non-negative (constructor invariant); subtract
            // by adding chrono::Duration::nanoseconds(-ns).
            let new_inst = a_inst
                .checked_sub_signed(chrono::Duration::nanoseconds(ns))
                .ok_or_else(|| RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "result-Instant in chrono representable range",
                    got: "out-of-range subtraction",
                })?;
            Ok(Value::Instant(new_inst))
        }
        Value::Instant(b_inst) => {
            // Instant - Instant -> Duration. Compute elapsed via
            // chrono's signed_duration_since; panic if negative
            // per §2.
            let dur = a_inst.signed_duration_since(b_inst);
            let ns = dur.num_nanoseconds().ok_or_else(|| {
                RuntimeError::TypeMismatch {
                    op: OP.into(),
                    expected: "elapsed nanoseconds in i64 range",
                    got: "out-of-range duration",
                }
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
        other => Err(RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "wat::time::Duration or wat::time::Instant",
            got: other.type_name(),
        }),
    }
}

/// `(:wat::time::+ instant duration) -> Instant`.
pub(crate) fn eval_time_add(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::+";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = eval(&args[0], env, sym)?;
    let b = eval(&args[1], env, sym)?;
    let a_inst = require_instant(OP, a)?;
    let ns = match b {
        Value::Duration(ns) => ns,
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: OP.into(),
                expected: "wat::time::Duration",
                got: other.type_name(),
            })
        }
    };
    let new_inst = a_inst
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        .ok_or_else(|| RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: "out-of-range addition",
        })?;
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::ago";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let ns = require_duration(OP, eval(&args[0], env, sym)?)?;
    let now = Utc::now();
    let result = now
        .checked_sub_signed(chrono::Duration::nanoseconds(ns))
        .ok_or_else(|| RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: "out-of-range subtraction",
        })?;
    Ok(Value::Instant(result))
}

/// `(:wat::time::from-now duration) -> :wat::time::Instant`. Equivalent
/// to `(:wat::time::+ (:wat::time::now) duration)`.
pub(crate) fn eval_time_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::time::from-now";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let ns = require_duration(OP, eval(&args[0], env, sym)?)?;
    let now = Utc::now();
    let result = now
        .checked_add_signed(chrono::Duration::nanoseconds(ns))
        .ok_or_else(|| RuntimeError::TypeMismatch {
            op: OP.into(),
            expected: "result-Instant in chrono representable range",
            got: "out-of-range addition",
        })?;
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
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?)?;
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
        .ok_or_else(|| RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: "out-of-range subtraction",
        })?;
    Ok(Value::Instant(result))
}

fn unit_from_now(
    op: &'static str,
    unit_nanos: i64,
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let n = require_i64(op, eval(&args[0], env, sym)?)?;
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
        .ok_or_else(|| RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "result-Instant in chrono representable range",
            got: "out-of-range addition",
        })?;
    Ok(Value::Instant(result))
}

// ─── Per-unit ago helpers ───────────────────────────────────────────

pub(crate) fn eval_time_nanoseconds_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::nanoseconds-ago", 1, args, env, sym)
}

pub(crate) fn eval_time_microseconds_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::microseconds-ago",
        NANOS_PER_MICRO,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_milliseconds_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::milliseconds-ago",
        NANOS_PER_MILLI,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_seconds_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::seconds-ago",
        NANOS_PER_SECOND,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_minutes_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(
        ":wat::time::minutes-ago",
        NANOS_PER_MINUTE,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_hours_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::hours-ago", NANOS_PER_HOUR, args, env, sym)
}

pub(crate) fn eval_time_days_ago(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_ago(":wat::time::days-ago", NANOS_PER_DAY, args, env, sym)
}

// ─── Per-unit from-now helpers ──────────────────────────────────────

pub(crate) fn eval_time_nanoseconds_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(":wat::time::nanoseconds-from-now", 1, args, env, sym)
}

pub(crate) fn eval_time_microseconds_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::microseconds-from-now",
        NANOS_PER_MICRO,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_milliseconds_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::milliseconds-from-now",
        NANOS_PER_MILLI,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_seconds_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::seconds-from-now",
        NANOS_PER_SECOND,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_minutes_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::minutes-from-now",
        NANOS_PER_MINUTE,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_hours_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(
        ":wat::time::hours-from-now",
        NANOS_PER_HOUR,
        args,
        env,
        sym,
    )
}

pub(crate) fn eval_time_days_from_now(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    unit_from_now(":wat::time::days-from-now", NANOS_PER_DAY, args, env, sym)
}

// ─── Helpers — local to this module ─────────────────────────────────

fn require_i64(op: &'static str, v: Value) -> Result<i64, RuntimeError> {
    match v {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "i64",
            got: other.type_name(),
        }),
    }
}

fn require_string(op: &'static str, v: Value) -> Result<String, RuntimeError> {
    match v {
        Value::String(s) => Ok((*s).clone()),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "String",
            got: other.type_name(),
        }),
    }
}

fn require_instant(op: &'static str, v: Value) -> Result<DateTime<Utc>, RuntimeError> {
    match v {
        Value::Instant(dt) => Ok(dt),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Instant",
            got: other.type_name(),
        }),
    }
}

fn require_duration(op: &'static str, v: Value) -> Result<i64, RuntimeError> {
    match v {
        Value::Duration(ns) => Ok(ns),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "wat::time::Duration",
            got: other.type_name(),
        }),
    }
}
