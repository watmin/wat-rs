;; :wat::telemetry::sqlite — reader-side surface (arc 093).
;;
;; The reader pairs with the existing writer (arc 091/096) to give
;; consumers an interrogation flow: open a frozen runs/*.db, stream
;; rows out via the substrate's `:wat::std::stream::*` circuit
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
;;   (LogCursor/step! cursor) -> :Option<:wat::telemetry::Event>
;;   (MetricCursor/step! cursor) -> :Option<:wat::telemetry::Event>
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
;; `Vec<TimeConstraint>`. They differ only in which time column
;; the cursor's prepared statement binds against (`time_ns` vs
;; `start_time_ns`) — the constraint enum doesn't need to know.
;;
;; AND-semantics across the vec. Empty vec = no narrowing
;; (full-table scan, slice-1 behavior preserved).
(:wat::core::enum :wat::telemetry::TimeConstraint
  (Since (instant :wat::time::Instant))
  (Until (instant :wat::time::Instant)))

;; Builders: one-line wraps around the variant constructors.
;; Reads more naturally at the call site than the variant form —
;; `(since (hours-ago 1))` vs
;; `(:wat::telemetry::TimeConstraint::Since (hours-ago 1))`.
(:wat::core::define
  (:wat::telemetry::since
    (instant :wat::time::Instant)
    -> :wat::telemetry::TimeConstraint)
  (:wat::telemetry::TimeConstraint::Since instant))

(:wat::core::define
  (:wat::telemetry::until
    (instant :wat::time::Instant)
    -> :wat::telemetry::TimeConstraint)
  (:wat::telemetry::TimeConstraint::Until instant))

;; ─── Cursor constructors (thin Rust forwarders) ────────────────

;; Cursor constructors. The constraint vec narrows the prepared
;; statement's WHERE clause; empty vec = full-table scan.
(:wat::core::define
  (:wat::telemetry::sqlite/log-cursor
    (handle :wat::sqlite::ReadHandle)
    (constraints :Vec<wat::telemetry::TimeConstraint>)
    -> :wat::telemetry::sqlite::LogCursor)
  (:rust::telemetry::sqlite::LogCursor::new handle constraints))

(:wat::core::define
  (:wat::telemetry::sqlite/metric-cursor
    (handle :wat::sqlite::ReadHandle)
    (constraints :Vec<wat::telemetry::TimeConstraint>)
    -> :wat::telemetry::sqlite::MetricCursor)
  (:rust::telemetry::sqlite::MetricCursor::new handle constraints))

(:wat::core::define
  (:wat::telemetry::sqlite::LogCursor/step!
    (cursor :wat::telemetry::sqlite::LogCursor)
    -> :Option<wat::telemetry::Event>)
  (:rust::telemetry::sqlite::LogCursor::step cursor))

(:wat::core::define
  (:wat::telemetry::sqlite::MetricCursor/step!
    (cursor :wat::telemetry::sqlite::MetricCursor)
    -> :Option<wat::telemetry::Event>)
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
;;   polymorphic Result<:T, :EvalError> shape) to lift it to a
;;   live Value of whatever type the log was. Caller annotates
;;   T at the binding site:
;;
;;     ((paper :Option<:trading::PaperResolved>)
;;      (:wat::telemetry::Event::Log/data-value e))
;;
;;   The lifted Value::Struct is what arc 098's :wat::form::matches?
;;   accepts as subject — the pry/gdb UX the arc 093 worked
;;   examples were designed around.
;;
;; Both return `:None` on the Metric variant (no data column).

(:wat::core::define
  (:wat::telemetry::Event::Log/data-ast
    (e :wat::telemetry::Event)
    -> :Option<wat::holon::HolonAST>)
  (:wat::core::match e -> :Option<wat::holon::HolonAST>
    ((:wat::telemetry::Event::Log _ _ _ _ _ _ data)
      (Some (:wat::edn::Tagged/0 data)))
    (_ :None)))

(:wat::core::define
  (:wat::telemetry::Event::Log/data-value<T>
    (e :wat::telemetry::Event)
    -> :Option<T>)
  (:wat::core::match e -> :Option<T>
    ((:wat::telemetry::Event::Log _ _ _ _ _ _ data)
      (:wat::core::match
        (:wat::eval-ast!
          (:wat::holon::to-watast (:wat::edn::Tagged/0 data)))
        -> :Option<T>
        ((Ok v) (Some v))
        ((Err _) :None)))
    (_ :None)))

;; ─── Stream sources via spawn-producer ─────────────────────────
;;
;; The producer-loop helpers iterate a cursor, sending each event
;; through the substrate channel until either:
;;   - the cursor returns :None (rows exhausted), or
;;   - the substrate Sender returns :None (consumer disconnected,
;;     drop-cascade has begun upstream).
;;
;; Each loop runs in the producer thread spawned by
;; :wat::std::stream::spawn-producer. Tail-recursive for unbounded
;; row counts.

(:wat::core::define
  (:wat::telemetry::sqlite/log-loop
    (cursor :wat::telemetry::sqlite::LogCursor)
    (tx :wat::kernel::QueueSender<wat::telemetry::Event>)
    -> :())
  (:wat::core::match
    (:wat::telemetry::sqlite::LogCursor/step! cursor)
    -> :()
    (:None ())
    ((Some event)
      (:wat::core::match
        (:wat::kernel::send tx event)
        -> :()
        (:None ())
        ((Some _)
          (:wat::telemetry::sqlite/log-loop cursor tx))))))

(:wat::core::define
  (:wat::telemetry::sqlite/metric-loop
    (cursor :wat::telemetry::sqlite::MetricCursor)
    (tx :wat::kernel::QueueSender<wat::telemetry::Event>)
    -> :())
  (:wat::core::match
    (:wat::telemetry::sqlite::MetricCursor/step! cursor)
    -> :()
    (:None ())
    ((Some event)
      (:wat::core::match
        (:wat::kernel::send tx event)
        -> :()
        (:None ())
        ((Some _)
          (:wat::telemetry::sqlite/metric-loop cursor tx))))))

;; (sqlite/stream-logs handle query) -> Stream<Event>
;;
;; Re-open the handle inside the producer thread (thread_owned
;; cells can't cross the spawn boundary; opening a fresh handle
;; from the captured path is cheap — sqlite handles many concurrent
;; read connections), construct a fresh cursor, drive the loop.
(:wat::core::define
  (:wat::telemetry::sqlite/stream-logs
    (handle :wat::sqlite::ReadHandle)
    (constraints :Vec<wat::telemetry::TimeConstraint>)
    -> :wat::std::stream::Stream<wat::telemetry::Event>)
  (:wat::core::let*
    (((path :String) (:wat::sqlite::ReadHandle/path handle)))
    (:wat::std::stream::spawn-producer
      (:wat::core::lambda
        ((tx :wat::kernel::QueueSender<wat::telemetry::Event>) -> :())
        (:wat::core::let*
          (((local-handle :wat::sqlite::ReadHandle)
            (:wat::sqlite::open-readonly path))
           ((cursor :wat::telemetry::sqlite::LogCursor)
            (:wat::telemetry::sqlite/log-cursor local-handle constraints)))
          (:wat::telemetry::sqlite/log-loop cursor tx))))))

(:wat::core::define
  (:wat::telemetry::sqlite/stream-metrics
    (handle :wat::sqlite::ReadHandle)
    (constraints :Vec<wat::telemetry::TimeConstraint>)
    -> :wat::std::stream::Stream<wat::telemetry::Event>)
  (:wat::core::let*
    (((path :String) (:wat::sqlite::ReadHandle/path handle)))
    (:wat::std::stream::spawn-producer
      (:wat::core::lambda
        ((tx :wat::kernel::QueueSender<wat::telemetry::Event>) -> :())
        (:wat::core::let*
          (((local-handle :wat::sqlite::ReadHandle)
            (:wat::sqlite::open-readonly path))
           ((cursor :wat::telemetry::sqlite::MetricCursor)
            (:wat::telemetry::sqlite/metric-cursor local-handle constraints)))
          (:wat::telemetry::sqlite/metric-loop cursor tx))))))
