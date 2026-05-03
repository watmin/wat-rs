//! Telemetry-side cursor types for arc 093's reader path.
//!
//! `LogCursor` / `MetricCursor` are wat values (one per
//! interrogation script) that drive a sqlite cursor row-by-row
//! and yield reified `:wat::telemetry::Event::Log` /
//! `Event::Metric` values. The wat-side `(stream-logs handle q)`
//! / `(stream-metrics handle q)` defines (slice 1d) wrap the
//! cursor in a `:wat::stream::spawn-producer` lambda; the
//! lambda calls `step!` per row and forwards through a bounded(1)
//! channel to whatever `filter` / `for-each` stage the user
//! composed downstream.
//!
//! # Thread model — three stages, two channels
//!
//! ```text
//! T2 (Rust)        T1 (wat lambda)              T0 (consumer)
//! ─────────        ───────────────              ────────────
//! sqlite step ──▶  cursor.rx (this module's
//! reify each row    bounded(1) channel)
//!                   step! pulls; forwards
//!                  ──▶  spawn-producer.tx     (filter / for-each)
//!                       (substrate channel)   ──▶  user code
//! ```
//!
//! T2 is the Rust producer thread spawned in this module's
//! constructor. It owns the rusqlite Connection + Statement +
//! Rows on its stack — the self-borrow lifetime that would block
//! storing them in a wat value is pushed entirely down here. T1
//! is the wat producer thread (substrate's `spawn-producer`
//! pattern). T0 is the consumer's main thread.
//!
//! Drop-cascade discipline: when T0 stops pulling (consumer
//! `for-each` returns / panics / drops its receiver), the
//! substrate channel closes; T1's send returns `:None` next
//! iteration, the lambda exits, T1 drops its end of THIS
//! module's channel; T2's send returns Err on next iteration,
//! the Rust thread breaks out of its row loop and exits cleanly.
//! Same shape as every other Stream<T> in the substrate.
//!
//! # Slice 1c surface
//!
//! - `LogCursor` / `MetricCursor` opaque types (registered
//!   manually via `RustSymbol`, not the `#[wat_dispatch]` macro,
//!   because the constructor takes a thread-owned ReadHandle as
//!   input and the step! method's `Option<Value>` return type
//!   sits outside what the macro currently emits cleanly).
//! - `(:wat::telemetry::sqlite/log-cursor handle query) -> LogCursor`
//! - `(:wat::telemetry::sqlite/metric-cursor handle query) -> MetricCursor`
//! - `(LogCursor/step! cursor) -> :Option<:wat::telemetry::Event::Log>`
//! - `(MetricCursor/step! cursor) -> :Option<:wat::telemetry::Event::Metric>`

use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, Receiver};
use rusqlite::{Connection, OpenFlags};
use wat::ast::WatAST;
use wat::edn_shim::{read_holon_ast_natural, read_holon_ast_tagged};
use wat::runtime::{
    eval, hashmap_key, Environment, EnumValue, RuntimeError, StructValue, SymbolTable, Value,
};
use wat::rust_deps::{
    downcast_ref_opaque, rust_opaque_arc, RustDispatch, RustScheme, RustSymbol, SchemeCtx,
    ThreadOwnedCell,
};
use wat::types::TypeExpr;

use wat_sqlite::ReadHandle;

// ─── Shared event type-path constant ──────────────────────────────

/// Substrate-defined Event enum. All reified rows produced by
/// these cursors carry this `type_path`; only the variant_name
/// (`"Log"` vs `"Metric"`) and field positions differ.
const EVENT_TYPE_PATH: &str = ":wat::telemetry::Event";

/// Type-path of `:wat::telemetry::TimeConstraint` (slice 2).
const TIME_CONSTRAINT_TYPE_PATH: &str = ":wat::telemetry::TimeConstraint";

// ─── Constraint → WHERE clause assembly ───────────────────────────

/// Parsed narrowing for a cursor's prepared statement. Slice 2
/// only ships time-range constraints; future slices may extend
/// this shape (composite predicates, IN-lists, etc.) — but the
/// substrate's stance is "anything that doesn't fit time-range
/// goes through the wat-side matcher" per arc 093 §6.
#[derive(Debug, Clone, Default)]
struct WhereClause {
    /// SQL fragment, including the leading " WHERE " when the
    /// vec was non-empty; empty string when no constraints.
    sql: String,
    /// Positional parameters bound at prepare time. One per `?`
    /// placeholder in `sql`. Slice 2: epoch-nanos i64s.
    params: Vec<i64>,
}

/// Walk a `:Vec<wat::telemetry::TimeConstraint>` value and build
/// a WhereClause against the cursor's time column. `time_col` is
/// `"time_ns"` for log cursors, `"start_time_ns"` for metric
/// cursors. Bare-string column name is safe — both values are
/// substrate-controlled, never user input.
fn parse_time_constraints(
    op: &'static str,
    time_col: &'static str,
    constraints: &Value,
) -> Result<WhereClause, RuntimeError> {
    let xs = match constraints {
        Value::Vec(xs) => xs.clone(),
        other => {
            return Err(RuntimeError::TypeMismatch {
                op: op.into(),
                expected: ":Vec<wat::telemetry::TimeConstraint>",
                got: other.type_name(),
                // arc 138: no span — parse_time_constraints receives &Value, no WatAST trace available
                span: wat::span::Span::unknown(),
            });
        }
    };
    let mut clauses: Vec<String> = Vec::with_capacity(xs.len());
    let mut params: Vec<i64> = Vec::with_capacity(xs.len());
    for (idx, v) in xs.iter().enumerate() {
        let ev = match v {
            Value::Enum(e) if e.type_path == TIME_CONSTRAINT_TYPE_PATH => e.clone(),
            other => {
                return Err(RuntimeError::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::telemetry::TimeConstraint",
                    got: other.type_name(),
                    // arc 138: no span — Vec element iteration over Values; per-element WatAST span unavailable
                    span: wat::span::Span::unknown(),
                });
            }
        };
        let instant = match ev.fields.first() {
            Some(Value::Instant(i)) => *i,
            _ => {
                return Err(RuntimeError::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "TimeConstraint::{} at index {idx} missing Instant field",
                        ev.variant_name
                    ),
                    // arc 138: no span — Vec element iteration over Values; per-element WatAST span unavailable
                    span: wat::span::Span::unknown(),
                });
            }
        };
        let nanos = instant.timestamp_nanos_opt().ok_or_else(|| {
            RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "TimeConstraint::{} at index {idx}: Instant out of i64-nanos range",
                    ev.variant_name
                ),
                // arc 138: no span — chrono range error on evaluated Instant value; no WatAST trace
                span: wat::span::Span::unknown(),
            }
        })?;
        let placeholder_idx = params.len() + 1;
        match ev.variant_name.as_str() {
            "Since" => clauses.push(format!("{time_col} >= ?{placeholder_idx}")),
            "Until" => clauses.push(format!("{time_col} <= ?{placeholder_idx}")),
            other => {
                return Err(RuntimeError::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "TimeConstraint variant {other}: only Since / Until are recognized"
                    ),
                    // arc 138: no span — Vec element iteration over Values; per-element WatAST span unavailable
                    span: wat::span::Span::unknown(),
                });
            }
        }
        params.push(nanos);
    }
    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    Ok(WhereClause { sql, params })
}

// ─── Cursor structs ───────────────────────────────────────────────

/// Receives reified `:wat::telemetry::Event::Log` values produced
/// by a Rust producer thread that owns the sqlite cursor on its
/// stack. `step!` pulls one event at a time; on disconnect (the
/// producer thread exits because rows are exhausted) `step!`
/// returns `:None`.
pub struct LogCursor {
    rx: Receiver<Value>,
    /// Stashed so dropping the cursor joins the producer thread.
    /// `Option` so we can take ownership in `Drop` (JoinHandle::join
    /// consumes self).
    join: Option<JoinHandle<()>>,
}

impl LogCursor {
    fn new(handle: &ReadHandle, narrowing: WhereClause) -> Self {
        let path = handle.path();
        let (tx, rx) = bounded::<Value>(1);
        let join = thread::Builder::new()
            .name("wat-telemetry-sqlite::LogCursor".into())
            .spawn(move || {
                drive_log_cursor(&path, narrowing, tx);
            })
            .expect("spawn LogCursor producer thread");
        Self { rx, join: Some(join) }
    }

    fn step(&self) -> Option<Value> {
        self.rx.recv().ok()
    }
}

impl Drop for LogCursor {
    fn drop(&mut self) {
        // Drop the receiver implicitly by returning; the producer
        // thread sees the disconnect on its next send and exits.
        // Then join to confirm clean shutdown. `take()` because
        // `JoinHandle::join` consumes self.
        if let Some(j) = self.join.take() {
            // Best-effort join — if the producer panicked we
            // surface nothing here; the panic already reached
            // wherever `recv()` was called from via the channel
            // disconnect. (Future arc could capture and re-raise.)
            let _ = j.join();
        }
    }
}

/// Mirror of [`LogCursor`] for metric rows.
pub struct MetricCursor {
    rx: Receiver<Value>,
    join: Option<JoinHandle<()>>,
}

impl MetricCursor {
    fn new(handle: &ReadHandle, narrowing: WhereClause) -> Self {
        let path = handle.path();
        let (tx, rx) = bounded::<Value>(1);
        let join = thread::Builder::new()
            .name("wat-telemetry-sqlite::MetricCursor".into())
            .spawn(move || {
                drive_metric_cursor(&path, narrowing, tx);
            })
            .expect("spawn MetricCursor producer thread");
        Self { rx, join: Some(join) }
    }

    fn step(&self) -> Option<Value> {
        self.rx.recv().ok()
    }
}

impl Drop for MetricCursor {
    fn drop(&mut self) {
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

// ─── Producer-thread functions ────────────────────────────────────

/// Run on T2. Owns Connection + Statement + Rows on the stack;
/// reifies each row; sends through `tx`. Exits on SQLITE_DONE
/// or on `tx.send` returning `Err` (consumer disconnect).
///
/// SQL shape: `SELECT … FROM log{narrowing.sql} ORDER BY time_ns
/// ASC`. With an empty narrowing this is the slice-1 full-table
/// scan; with `Since` / `Until` constraints the time index narrows
/// the candidate set in O(log N) before the wat-side matcher
/// runs.
fn drive_log_cursor(
    path: &str,
    narrowing: WhereClause,
    tx: crossbeam_channel::Sender<Value>,
) {
    let conn = match Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => panic!(
            "wat-telemetry-sqlite::LogCursor: re-open {path} for producer thread: {e}"
        ),
    };
    let sql = format!(
        "SELECT time_ns, namespace, caller, level, uuid, tags, data \
         FROM log{} ORDER BY time_ns ASC",
        narrowing.sql
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => panic!(
            "wat-telemetry-sqlite::LogCursor: prepare {sql:?}: {e}"
        ),
    };
    let bound: Vec<&dyn rusqlite::ToSql> =
        narrowing.params.iter().map(|n| n as &dyn rusqlite::ToSql).collect();
    let mut rows = match stmt.query(bound.as_slice()) {
        Ok(r) => r,
        Err(e) => panic!(
            "wat-telemetry-sqlite::LogCursor: query {sql:?}: {e}"
        ),
    };
    loop {
        match rows.next() {
            Ok(Some(row)) => match reify_log_row(row) {
                Ok(v) => {
                    if tx.send(v).is_err() {
                        // consumer disconnected — drop-cascade
                        return;
                    }
                }
                Err(e) => panic!(
                    "wat-telemetry-sqlite::LogCursor: row reify failed: {e}"
                ),
            },
            Ok(None) => return,
            Err(e) => panic!(
                "wat-telemetry-sqlite::LogCursor: rows.next failed: {e}"
            ),
        }
    }
}

/// Twin of `drive_log_cursor` for the `metric` table. Sort by
/// `start_time_ns` ASC.
fn drive_metric_cursor(
    path: &str,
    narrowing: WhereClause,
    tx: crossbeam_channel::Sender<Value>,
) {
    let conn = match Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => panic!(
            "wat-telemetry-sqlite::MetricCursor: re-open {path} for producer thread: {e}"
        ),
    };
    let sql = format!(
        "SELECT start_time_ns, end_time_ns, namespace, uuid, tags, metric_name, metric_value, metric_unit \
         FROM metric{} ORDER BY start_time_ns ASC",
        narrowing.sql
    );
    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => panic!(
            "wat-telemetry-sqlite::MetricCursor: prepare {sql:?}: {e}"
        ),
    };
    let bound: Vec<&dyn rusqlite::ToSql> =
        narrowing.params.iter().map(|n| n as &dyn rusqlite::ToSql).collect();
    let mut rows = match stmt.query(bound.as_slice()) {
        Ok(r) => r,
        Err(e) => panic!(
            "wat-telemetry-sqlite::MetricCursor: query {sql:?}: {e}"
        ),
    };
    loop {
        match rows.next() {
            Ok(Some(row)) => match reify_metric_row(row) {
                Ok(v) => {
                    if tx.send(v).is_err() {
                        return;
                    }
                }
                Err(e) => panic!(
                    "wat-telemetry-sqlite::MetricCursor: row reify failed: {e}"
                ),
            },
            Ok(None) => return,
            Err(e) => panic!(
                "wat-telemetry-sqlite::MetricCursor: rows.next failed: {e}"
            ),
        }
    }
}

// ─── Row reify ────────────────────────────────────────────────────

/// Decode one `log` row into `:wat::telemetry::Event::Log`.
/// Field order (positional, per the variant declaration):
///   0: time-ns (i64)
///   1: namespace (NoTag → HolonAST)
///   2: caller (NoTag → HolonAST)
///   3: level (NoTag → HolonAST)
///   4: uuid (String)
///   5: tags (HashMap<HolonAST,HolonAST>)
///   6: data (Tagged → HolonAST round-trip-safe)
fn reify_log_row(row: &rusqlite::Row<'_>) -> Result<Value, ReifyError> {
    let time_ns: i64 = row.get(0).map_err(|e| ReifyError::ColumnRead("time_ns", e))?;
    let namespace: String = row.get(1).map_err(|e| ReifyError::ColumnRead("namespace", e))?;
    let caller: String = row.get(2).map_err(|e| ReifyError::ColumnRead("caller", e))?;
    let level: String = row.get(3).map_err(|e| ReifyError::ColumnRead("level", e))?;
    let uuid: String = row.get(4).map_err(|e| ReifyError::ColumnRead("uuid", e))?;
    let tags: String = row.get(5).map_err(|e| ReifyError::ColumnRead("tags", e))?;
    let data: String = row.get(6).map_err(|e| ReifyError::ColumnRead("data", e))?;

    Ok(make_event(
        "Log",
        vec![
            Value::i64(time_ns),
            decode_notag_holon(&namespace, "namespace")?,
            decode_notag_holon(&caller, "caller")?,
            decode_notag_holon(&level, "level")?,
            Value::String(Arc::new(uuid)),
            decode_tags(&tags)?,
            decode_tagged_holon(&data, "data")?,
        ],
    ))
}

/// Decode one `metric` row into `:wat::telemetry::Event::Metric`.
/// Field order (positional, per the variant declaration):
///   0: start-time-ns (i64)
///   1: end-time-ns (i64)
///   2: namespace (NoTag)
///   3: uuid (String)
///   4: tags (HashMap)
///   5: metric-name (NoTag)
///   6: metric-value (NoTag)
///   7: metric-unit (NoTag)
fn reify_metric_row(row: &rusqlite::Row<'_>) -> Result<Value, ReifyError> {
    let start_time_ns: i64 = row.get(0).map_err(|e| ReifyError::ColumnRead("start_time_ns", e))?;
    let end_time_ns: i64 = row.get(1).map_err(|e| ReifyError::ColumnRead("end_time_ns", e))?;
    let namespace: String = row.get(2).map_err(|e| ReifyError::ColumnRead("namespace", e))?;
    let uuid: String = row.get(3).map_err(|e| ReifyError::ColumnRead("uuid", e))?;
    let tags: String = row.get(4).map_err(|e| ReifyError::ColumnRead("tags", e))?;
    let metric_name: String = row.get(5).map_err(|e| ReifyError::ColumnRead("metric_name", e))?;
    let metric_value: String = row.get(6).map_err(|e| ReifyError::ColumnRead("metric_value", e))?;
    let metric_unit: String = row.get(7).map_err(|e| ReifyError::ColumnRead("metric_unit", e))?;

    Ok(make_event(
        "Metric",
        vec![
            Value::i64(start_time_ns),
            Value::i64(end_time_ns),
            decode_notag_holon(&namespace, "namespace")?,
            Value::String(Arc::new(uuid)),
            decode_tags(&tags)?,
            decode_notag_holon(&metric_name, "metric_name")?,
            decode_notag_holon(&metric_value, "metric_value")?,
            decode_notag_holon(&metric_unit, "metric_unit")?,
        ],
    ))
}

/// Construct a `Value::Enum` for `:wat::telemetry::Event/<variant>`.
fn make_event(variant: &str, fields: Vec<Value>) -> Value {
    Value::Enum(Arc::new(EnumValue {
        type_path: EVENT_TYPE_PATH.into(),
        variant_name: variant.into(),
        fields,
    }))
}

/// Decode a `:wat::edn::NoTag` TEXT column to its runtime value:
/// `Value::Struct(":wat::edn::NoTag", [Value::holon__HolonAST(ast)])`.
fn decode_notag_holon(text: &str, col: &'static str) -> Result<Value, ReifyError> {
    let ast = read_holon_ast_natural(text).map_err(|e| ReifyError::DecodeNoTag(col, e.to_string()))?;
    Ok(wrap_newtype(":wat::edn::NoTag", Value::holon__HolonAST(ast)))
}

/// Decode a `:wat::edn::Tagged` TEXT column to its runtime value:
/// `Value::Struct(":wat::edn::Tagged", [Value::holon__HolonAST(ast)])`.
/// Uses the round-trip-safe `#wat-edn.holon/*` tagged read.
fn decode_tagged_holon(text: &str, col: &'static str) -> Result<Value, ReifyError> {
    let ast = read_holon_ast_tagged(text).map_err(|e| ReifyError::DecodeTagged(col, e.to_string()))?;
    Ok(wrap_newtype(":wat::edn::Tagged", Value::holon__HolonAST(ast)))
}

/// Decode the `tags` TEXT column (a tagless EDN map) to a
/// `Value::wat__std__HashMap` of HolonAST keys / HolonAST values.
fn decode_tags(text: &str) -> Result<Value, ReifyError> {
    let edn = wat_edn::parse_owned(text)
        .map_err(|e| ReifyError::DecodeTags(format!("EDN parse: {e}")))?;
    let entries = match &edn {
        wat_edn::OwnedValue::Map(es) => es,
        other => {
            return Err(ReifyError::DecodeTags(format!(
                "expected EDN Map for tags column; got {other:?}"
            )));
        }
    };
    let mut map: std::collections::HashMap<String, (Value, Value)> =
        std::collections::HashMap::with_capacity(entries.len());
    for (k, v) in entries {
        // Each k / v in the writer-side natural map IS a HolonAST.
        let k_ast = wat::edn_shim::read_holon_ast_natural(&wat_edn::write(k))
            .map_err(|e| ReifyError::DecodeTags(format!("key: {e}")))?;
        let v_ast = wat::edn_shim::read_holon_ast_natural(&wat_edn::write(v))
            .map_err(|e| ReifyError::DecodeTags(format!("value: {e}")))?;
        let k_val = Value::holon__HolonAST(k_ast);
        let v_val = Value::holon__HolonAST(v_ast);
        let canonical = hashmap_key(":wat::telemetry::sqlite/tags-decode", &k_val)
            .map_err(|e| ReifyError::DecodeTags(format!("hashmap_key: {e}")))?;
        map.insert(canonical, (k_val, v_val));
    }
    Ok(Value::wat__std__HashMap(Arc::new(map)))
}

/// Build a newtype `Value::Struct` wrapping an inner value.
/// Mirrors the writer-side construction `(:wat::edn::NoTag/new …)`
/// produces. The newtype's runtime shape is arity-1 tuple struct
/// (per arc 049's `register_newtype_methods`); we replicate that
/// shape here for read-side reify.
fn wrap_newtype(type_name: &str, inner: Value) -> Value {
    Value::Struct(Arc::new(StructValue {
        type_name: type_name.into(),
        fields: vec![inner],
    }))
}

#[derive(Debug)]
enum ReifyError {
    ColumnRead(&'static str, rusqlite::Error),
    DecodeNoTag(&'static str, String),
    DecodeTagged(&'static str, String),
    DecodeTags(String),
}

impl std::fmt::Display for ReifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReifyError::ColumnRead(col, e) => write!(f, "column {col}: {e}"),
            ReifyError::DecodeNoTag(col, e) => write!(f, "NoTag decode of {col}: {e}"),
            ReifyError::DecodeTagged(col, e) => write!(f, "Tagged decode of {col}: {e}"),
            ReifyError::DecodeTags(e) => write!(f, "tags map decode: {e}"),
        }
    }
}

// ─── Manual RustSymbol registration ───────────────────────────────
//
// The cursor primitives don't fit `#[wat_dispatch]` cleanly:
// - Constructors take a thread-owned ReadHandle (a wat Value)
//   and need to call `.path()` through the ThreadOwnedCell
//   wrapper — the macro doesn't model cross-shim borrows.
// - `step!` returns `:Option<:T>` where `T` is a user-declared
//   enum variant; the macro emits owned-type returns, not
//   `Option`-wrapped opaque payloads.
//
// Hand-rolled registration matches the auto-spawn shim shape in
// `auto.rs` (`RustSymbol { path, dispatch, scheme }` literals
// with fn-pointer casts). Four primitives ship in slice 1:
// log-cursor / metric-cursor constructors, plus
// LogCursor::step! / MetricCursor::step!.

const LOG_CURSOR_PATH: &str = ":rust::telemetry::sqlite::LogCursor";
const METRIC_CURSOR_PATH: &str = ":rust::telemetry::sqlite::MetricCursor";
const READ_HANDLE_PATH: &str = ":rust::sqlite::ReadHandle";

pub(crate) fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    builder.register_symbol(RustSymbol {
        path: ":rust::telemetry::sqlite::LogCursor::new",
        dispatch: dispatch_log_cursor_new as RustDispatch,
        scheme: scheme_log_cursor_new as RustScheme,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::telemetry::sqlite::MetricCursor::new",
        dispatch: dispatch_metric_cursor_new as RustDispatch,
        scheme: scheme_metric_cursor_new as RustScheme,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::telemetry::sqlite::LogCursor::step",
        dispatch: dispatch_log_cursor_step as RustDispatch,
        scheme: scheme_cursor_step as RustScheme,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::telemetry::sqlite::MetricCursor::step",
        dispatch: dispatch_metric_cursor_step as RustDispatch,
        scheme: scheme_cursor_step as RustScheme,
    });
    // Type declarations so `:wat::core::use!` accepts the path.
    builder.register_type(wat::rust_deps::RustTypeDecl { path: LOG_CURSOR_PATH });
    builder.register_type(wat::rust_deps::RustTypeDecl { path: METRIC_CURSOR_PATH });
}

// ── Schemes ─────────────────────────────────────────────────────────

fn scheme_log_cursor_new(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    scheme_cursor_new_inner(
        args,
        ctx,
        ":rust::telemetry::sqlite::LogCursor::new",
        LOG_CURSOR_PATH,
    )
}

fn scheme_metric_cursor_new(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    scheme_cursor_new_inner(
        args,
        ctx,
        ":rust::telemetry::sqlite::MetricCursor::new",
        METRIC_CURSOR_PATH,
    )
}

fn scheme_cursor_new_inner(
    args: &[WatAST],
    ctx: &mut dyn SchemeCtx,
    op: &'static str,
    cursor_path: &'static str,
) -> Option<TypeExpr> {
    if args.len() != 2 {
        // Pattern B: arity mismatch — use first arg span if any, else unknown
        ctx.push_arity_mismatch(
            op,
            2,
            args.len(),
            args.first()
                .map(|a| a.span().clone())
                .unwrap_or_else(wat::span::Span::unknown),
        );
        return Some(TypeExpr::Path(cursor_path.into()));
    }
    // #1 — :wat::sqlite::ReadHandle
    if let Some(t) = ctx.infer(&args[0]) {
        let expected = TypeExpr::Path(READ_HANDLE_PATH.into());
        if !ctx.unify_types(&t, &expected) {
            // Pattern A: type mismatch on arg #1
            ctx.push_type_mismatch(
                op,
                "#1",
                READ_HANDLE_PATH.into(),
                format!("{:?}", ctx.apply_subst(&t)),
                args[0].span().clone(),
            );
        }
    }
    // #2 — :Vec<:wat::telemetry::TimeConstraint>
    if let Some(t) = ctx.infer(&args[1]) {
        let expected = TypeExpr::Parametric {
            head: "Vec".into(),
            args: vec![TypeExpr::Path(TIME_CONSTRAINT_TYPE_PATH.into())],
        };
        if !ctx.unify_types(&t, &expected) {
            // Pattern A: type mismatch on arg #2
            ctx.push_type_mismatch(
                op,
                "#2",
                ":Vec<wat::telemetry::TimeConstraint>".into(),
                format!("{:?}", ctx.apply_subst(&t)),
                args[1].span().clone(),
            );
        }
    }
    Some(TypeExpr::Path(cursor_path.into()))
}

fn scheme_cursor_step(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != 1 {
        // Pattern B: arity mismatch — use first arg span if any, else unknown
        ctx.push_arity_mismatch(
            ":rust::telemetry::sqlite::*::step",
            1,
            args.len(),
            args.first()
                .map(|a| a.span().clone())
                .unwrap_or_else(wat::span::Span::unknown),
        );
    }
    // Return :Option<:wat::telemetry::Event>. Both Log and
    // Metric cursors return events of the same enum type — the
    // user pattern-matches the variant at the call site.
    Some(TypeExpr::Parametric {
        head: "Option".into(),
        args: vec![TypeExpr::Path(EVENT_TYPE_PATH.into())],
    })
}

// ── Dispatchers ─────────────────────────────────────────────────────

fn dispatch_log_cursor_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::telemetry::sqlite::LogCursor::new";
    let (handle, narrowing) = eval_handle_and_constraints(args, env, sym, OP, "time_ns")?;
    let cursor = LogCursor::new(&handle, narrowing);
    Ok(wat::rust_deps::make_rust_opaque(
        LOG_CURSOR_PATH,
        ThreadOwnedCell::new(cursor),
    ))
}

fn dispatch_metric_cursor_new(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::telemetry::sqlite::MetricCursor::new";
    let (handle, narrowing) = eval_handle_and_constraints(args, env, sym, OP, "start_time_ns")?;
    let cursor = MetricCursor::new(&handle, narrowing);
    Ok(wat::rust_deps::make_rust_opaque(
        METRIC_CURSOR_PATH,
        ThreadOwnedCell::new(cursor),
    ))
}

/// Eval the cursor-constructor args as `(ReadHandle,
/// Vec<TimeConstraint>)`. Returns a fresh ReadHandle (re-opened
/// from the path stash; the original cell lives in the caller's
/// thread and can't cross the spawn boundary) plus a parsed
/// WhereClause keyed against the cursor's time column.
fn eval_handle_and_constraints(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op: &'static str,
    time_col: &'static str,
) -> Result<(ReadHandle, WhereClause), RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 2,
            got: args.len(),
            // arc 138: no span — eval_handle_and_constraints has no list_span; cross-file broadening out of scope
            span: wat::span::Span::unknown(),
        });
    }
    let handle_val = eval(&args[0], env, sym)?;
    let inner = rust_opaque_arc(&handle_val, READ_HANDLE_PATH, op)?;
    let cell: &ThreadOwnedCell<ReadHandle> = downcast_ref_opaque(&inner, READ_HANDLE_PATH, op)?;
    let path = cell.with_ref(op, |h| h.path())?;
    let handle = ReadHandle::open(path);

    let constraints_val = eval(&args[1], env, sym)?;
    let narrowing = parse_time_constraints(op, time_col, &constraints_val)?;
    Ok((handle, narrowing))
}

fn dispatch_log_cursor_step(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let result = with_cursor_step::<LogCursor>(
        args,
        env,
        sym,
        ":rust::telemetry::sqlite::LogCursor::step",
        LOG_CURSOR_PATH,
        |c| c.step(),
    )?;
    Ok(Value::Option(Arc::new(result)))
}

fn dispatch_metric_cursor_step(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    let result = with_cursor_step::<MetricCursor>(
        args,
        env,
        sym,
        ":rust::telemetry::sqlite::MetricCursor::step",
        METRIC_CURSOR_PATH,
        |c| c.step(),
    )?;
    Ok(Value::Option(Arc::new(result)))
}

fn with_cursor_step<C: Send + Sync + 'static>(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    op: &'static str,
    cursor_path: &'static str,
    step: impl FnOnce(&C) -> Option<Value>,
) -> Result<Option<Value>, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: op.into(),
            expected: 1,
            got: args.len(),
            // arc 138: no span — with_cursor_step has no list_span; cross-file broadening out of scope
            span: wat::span::Span::unknown(),
        });
    }
    let cur_val = eval(&args[0], env, sym)?;
    let inner = rust_opaque_arc(&cur_val, cursor_path, op)?;
    let cell: &ThreadOwnedCell<C> = downcast_ref_opaque(&inner, cursor_path, op)?;
    cell.with_ref(op, step)
}
