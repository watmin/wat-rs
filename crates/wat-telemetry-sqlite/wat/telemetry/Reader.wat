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

;; ─── Query stubs (slice 1) ──────────────────────────────────────
;;
;; Slice 2 will replace these with constraint-vec structs:
;;
;;   (:wat::core::struct :wat::telemetry::LogQuery
;;     (constraints :Vec<wat::telemetry::LogConstraint>))
;;
;; For slice 1 they're empty unit-shape types so the cursor
;; constructor signature is forward-compatible — call sites pass
;; an empty query and slice 2 changes the field set without
;; rewriting them.
(:wat::core::struct :wat::telemetry::LogQuery)
(:wat::core::struct :wat::telemetry::MetricQuery)

;; ─── Cursor constructors (thin Rust forwarders) ────────────────

;; Slice 1 ignores the query (full-table scan); slice 2 threads
;; constraints into the prepared statement's WHERE clause.
(:wat::core::define
  (:wat::telemetry::sqlite/log-cursor
    (handle :wat::sqlite::ReadHandle)
    (_query :wat::telemetry::LogQuery)
    -> :wat::telemetry::sqlite::LogCursor)
  (:rust::telemetry::sqlite::LogCursor::new handle))

(:wat::core::define
  (:wat::telemetry::sqlite/metric-cursor
    (handle :wat::sqlite::ReadHandle)
    (_query :wat::telemetry::MetricQuery)
    -> :wat::telemetry::sqlite::MetricCursor)
  (:rust::telemetry::sqlite::MetricCursor::new handle))

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
    (tx :rust::crossbeam_channel::Sender<wat::telemetry::Event>)
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
    (tx :rust::crossbeam_channel::Sender<wat::telemetry::Event>)
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
    (query :wat::telemetry::LogQuery)
    -> :wat::std::stream::Stream<wat::telemetry::Event>)
  (:wat::core::let*
    (((path :String) (:wat::sqlite::ReadHandle/path handle)))
    (:wat::std::stream::spawn-producer
      (:wat::core::lambda
        ((tx :rust::crossbeam_channel::Sender<wat::telemetry::Event>) -> :())
        (:wat::core::let*
          (((local-handle :wat::sqlite::ReadHandle)
            (:wat::sqlite::open-readonly path))
           ((cursor :wat::telemetry::sqlite::LogCursor)
            (:wat::telemetry::sqlite/log-cursor local-handle query)))
          (:wat::telemetry::sqlite/log-loop cursor tx))))))

(:wat::core::define
  (:wat::telemetry::sqlite/stream-metrics
    (handle :wat::sqlite::ReadHandle)
    (query :wat::telemetry::MetricQuery)
    -> :wat::std::stream::Stream<wat::telemetry::Event>)
  (:wat::core::let*
    (((path :String) (:wat::sqlite::ReadHandle/path handle)))
    (:wat::std::stream::spawn-producer
      (:wat::core::lambda
        ((tx :rust::crossbeam_channel::Sender<wat::telemetry::Event>) -> :())
        (:wat::core::let*
          (((local-handle :wat::sqlite::ReadHandle)
            (:wat::sqlite::open-readonly path))
           ((cursor :wat::telemetry::sqlite::MetricCursor)
            (:wat::telemetry::sqlite/metric-cursor local-handle query)))
          (:wat::telemetry::sqlite/metric-loop cursor tx))))))
