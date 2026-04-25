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
