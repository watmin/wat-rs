;; :wat::telemetry::sqlite — reader-side surface (arc 093).
;;
;; The reader pairs with the existing writer (arc 091/096) to give
;; consumers an interrogation flow: open a frozen runs/*.db, stream
;; rows out via the substrate's `:wat::stream::*` circuit
;; pattern, filter / for-each in wat. Three stages, two bounded(1)
;; channels, drop-cascade shutdown — exactly the existing
;; spawn-producer model.
;;
;; Slice 1 surface:
;;
;;   :wat::telemetry::sqlite::LogCursor / MetricCursor — typealiases
;;     of the Rust shim opaque types in cursor.rs. Each cursor wraps
;;     a Rust producer thread that owns the rusqlite Connection +
;;     Statement + Rows on its stack, sending reified Event variants
;;     through an internal bounded(1) channel.
;;
;;   :wat::telemetry::LogQuery / MetricQuery — slice 1 stubs. Empty
;;     unit-shape structs; slice 2 will populate them with
;;     Since/Until variants (low-cardinality time-range pushdown
;;     into SQL — every other predicate filters in wat per arc 093 §6).
;;
;;   (sqlite/log-cursor handle query) -> LogCursor
;;   (sqlite/metric-cursor handle query) -> MetricCursor
;;     Thin wrappers around the Rust constructors. Slice 1 ignores
;;     the query argument (full-table scan, ORDER BY time_ns ASC).
;;
;;   (LogCursor/step! cursor) -> :wat::core::Option<:wat::telemetry::Event>
;;   (MetricCursor/step! cursor) -> :wat::core::Option<:wat::telemetry::Event>
;;     Pull one event from the cursor. :None on exhaustion.
;;
;;   (sqlite/stream-logs handle query) -> Stream<Event>
;;   (sqlite/stream-metrics handle query) -> Stream<Event>
;;     Compose spawn-producer around the cursor: re-open the handle
;;     inside the producer thread (thread_owned discipline forbids
;;     transferring the original handle across the spawn boundary),
;;     create a cursor, loop step!→send until either side hits :None.

(:wat::core::use! :rust::telemetry::sqlite::LogCursor)
(:wat::core::use! :rust::telemetry::sqlite::MetricCursor)

(:wat::core::typealias :wat::telemetry::sqlite::LogCursor
  :rust::telemetry::sqlite::LogCursor)

(:wat::core::typealias :wat::telemetry::sqlite::MetricCursor
  :rust::telemetry::sqlite::MetricCursor)

;; ─── TimeConstraint (slice 2) ──────────────────────────────────
;;
;; The only constraints the SQL layer accepts are time-range —
;; `Since(Instant)` (renders as `time_col >= ?`) and
;; `Until(Instant)` (renders as `time_col <= ?`). Every other
;; predicate (namespace, uuid, level, caller, metric_name, tags,
;; data) filters in wat via stream + matches? per arc 093 §6's
;; line in the sand.
;;
;; Both stream-logs and stream-metrics consume the SAME
;; `wat::core::Vector<TimeConstraint>`. They differ only in which time column
;; the cursor's prepared statement binds against (`time_ns` vs
;; `start_time_ns`) — the constraint enum doesn't need to know.
;;
;; AND-semantics across the vec. Empty vec = no narrowing
;; (full-table scan, slice-1 behavior preserved).
(:wat::core::defenum :wat::telemetry::TimeConstraint :wat::enum::Pure
  :Since [instant <- :wat::time::Instant]
  :Until [instant <- :wat::time::Instant])

;; Builders: one-line wraps around the variant constructors.
;; Reads more naturally at the call site than the variant form —
;; `(since (hours-ago 1))` vs
;; `(:wat::telemetry::TimeConstraint::Since (hours-ago 1))`.
(:wat::core::defn :wat::telemetry::since
  [instant <- :wat::time::Instant]
  -> :wat::telemetry::TimeConstraint
  (:wat::telemetry::TimeConstraint::Since instant))

(:wat::core::defn :wat::telemetry::until
  [instant <- :wat::time::Instant]
  -> :wat::telemetry::TimeConstraint
  (:wat::telemetry::TimeConstraint::Until instant))

;; ─── Cursor constructors (thin Rust forwarders) ────────────────

;; Cursor constructors. The constraint vec narrows the prepared
;; statement's WHERE clause; empty vec = full-table scan.
(:wat::core::defn :wat::telemetry::sqlite/log-cursor
  [handle      <- :wat::sqlite::ReadHandle
   constraints <- :wat::core::Vector<wat::telemetry::TimeConstraint>]
  -> :wat::telemetry::sqlite::LogCursor
  (:rust::telemetry::sqlite::LogCursor::new handle constraints))

(:wat::core::defn :wat::telemetry::sqlite/metric-cursor
  [handle      <- :wat::sqlite::ReadHandle
   constraints <- :wat::core::Vector<wat::telemetry::TimeConstraint>]
  -> :wat::telemetry::sqlite::MetricCursor
  (:rust::telemetry::sqlite::MetricCursor::new handle constraints))

(:wat::core::defn :wat::telemetry::sqlite::LogCursor/step!
  [cursor <- :wat::telemetry::sqlite::LogCursor]
  -> :wat::core::Option<wat::telemetry::Event>
  (:rust::telemetry::sqlite::LogCursor::step cursor))

(:wat::core::defn :wat::telemetry::sqlite::MetricCursor/step!
  [cursor <- :wat::telemetry::sqlite::MetricCursor]
  -> :wat::core::Option<wat::telemetry::Event>
  (:rust::telemetry::sqlite::MetricCursor::step cursor))

;; ─── Event::Log/data-ast / data-value (slice 3) ─────────────
;;
;; Materialization helpers that bridge a streamed Event back to
;; the shape it was logged at:
;;
;; - `data-ast` extracts the raw HolonAST from the Tagged data
;;   column. Cheap: pattern-match + newtype unwrap. Use when
;;   you want to grep the AST shape directly (e.g., "did this
;;   log carry a Bind structure?").
;; - `data-value<T>` runs the AST through eval-ast! (arc 102's
;;   polymorphic wat::core::Result<:T, :EvalError> shape) to lift it to a
;;   live Value of whatever type the log was. Caller annotates
;;   T at the binding site:
;;
;;     ((paper :wat::core::Option<:trading::PaperResolved>)
;;      (:wat::telemetry::Event::Log/data-value e))
;;
;;   The lifted Value::Struct is what arc 098's :wat::form::matches?
;;   accepts as subject — the pry/gdb UX the arc 093 worked
;;   examples were designed around.
;;
;; Both return `:None` on the Metric variant (no data column).

(:wat::core::defn :wat::telemetry::Event::Log/data-ast
  [e <- :wat::telemetry::Event]
  -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::match e -> :wat::core::Option<wat::holon::HolonAST>
    ((:wat::telemetry::Event::Log _ _ _ _ _ _ data)
      (:wat::core::Some (:wat::edn::Tagged/0 data)))
    (_ :wat::core::None)))

(:wat::core::defn :wat::telemetry::Event::Log/data-value<T>
  [e <- :wat::telemetry::Event]
  -> :wat::core::Option<T>
  (:wat::core::match e -> :wat::core::Option<T>
    ((:wat::telemetry::Event::Log _ _ _ _ _ _ data)
      (:wat::core::match
        (:wat::eval-ast!
          (:wat::holon::to-wat (:wat::edn::Tagged/0 data)))
        -> :wat::core::Option<T>
        ((:wat::core::Ok v) (:wat::core::Some v))
        ((:wat::core::Err _) :wat::core::None)))
    (_ :wat::core::None)))

;; ─── Eager read sources ─────────────────────────────────────────
;;
;; Arc 118 (2026-06-27): migrated off the annihilated `:wat::stream::*`
;; (thread-per-stage, built wrong). A telemetry read returns BOUNDED
;; query results, so the honest shape is an eager `Vector<Event>`, not
;; a thread-backed stream. The loop helpers iterate a cursor and
;; accumulate each event into `acc` until the cursor returns :None
;; (rows exhausted). Tail-recursive; no thread, no channel.

(:wat::core::defn :wat::telemetry::sqlite/log-loop
  [cursor <- :wat::telemetry::sqlite::LogCursor
   acc    <- :wat::core::Vector<wat::telemetry::Event>]
  -> :wat::core::Vector<wat::telemetry::Event>
  (:wat::core::match
    (:wat::telemetry::sqlite::LogCursor/step! cursor)
    -> :wat::core::Vector<wat::telemetry::Event>
    (:wat::core::None acc)
    ((:wat::core::Some event)
      (:wat::telemetry::sqlite/log-loop cursor (:wat::core::conj acc event)))))

(:wat::core::defn :wat::telemetry::sqlite/metric-loop
  [cursor <- :wat::telemetry::sqlite::MetricCursor
   acc    <- :wat::core::Vector<wat::telemetry::Event>]
  -> :wat::core::Vector<wat::telemetry::Event>
  (:wat::core::match
    (:wat::telemetry::sqlite::MetricCursor/step! cursor)
    -> :wat::core::Vector<wat::telemetry::Event>
    (:wat::core::None acc)
    ((:wat::core::Some event)
      (:wat::telemetry::sqlite/metric-loop cursor (:wat::core::conj acc event)))))

;; (sqlite/read-logs handle query) -> Vector<Event>
;;
;; Open a fresh read-only cursor over the handle's path and drive the
;; loop to exhaustion, accumulating into a Vector. Eager: the result is
;; the full bounded query result. (Arc 118 — was stream-logs over the
;; annihilated :wat::stream::*.)
(:wat::core::defn :wat::telemetry::sqlite/read-logs
  [handle      <- :wat::sqlite::ReadHandle
   constraints <- :wat::core::Vector<wat::telemetry::TimeConstraint>]
  -> :wat::core::Vector<wat::telemetry::Event>
  (:wat::core::let
    [local-handle (:wat::sqlite::open-readonly (:wat::sqlite::ReadHandle/path handle))
     cursor       (:wat::telemetry::sqlite/log-cursor local-handle constraints)]
    (:wat::telemetry::sqlite/log-loop cursor (:wat::core::Vector :wat::telemetry::Event))))

(:wat::core::defn :wat::telemetry::sqlite/read-metrics
  [handle      <- :wat::sqlite::ReadHandle
   constraints <- :wat::core::Vector<wat::telemetry::TimeConstraint>]
  -> :wat::core::Vector<wat::telemetry::Event>
  (:wat::core::let
    [local-handle (:wat::sqlite::open-readonly (:wat::sqlite::ReadHandle/path handle))
     cursor       (:wat::telemetry::sqlite/metric-cursor local-handle constraints)]
    (:wat::telemetry::sqlite/metric-loop cursor (:wat::core::Vector :wat::telemetry::Event))))
