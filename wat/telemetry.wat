;; wat/telemetry.wat — arc 278 stone ①: the `:wat::telemetry'` DATA VOCABULARY (core baked source).
;;
;; The telemetry facility's structural foundation: the enums/typealias/surfaces/records that the
;; sink + producer services (later stones) build on. First real consumer of surface-splice
;; (shipped 4c98b2ef): `Metric`/`Log` are `defrecord`s that SPLICE the `Scope` surface via
;; `~@:wat::telemetry'::Scope` — the 4 Scope fields inline (splice-first) before each record's own
;; fields, and the unified aggregate ctor + register_aggregate_methods mint the spliced accessors
;; (e.g. `Metric/namespace`) for free.
;;
;; The namespace is PRIMED (`:wat::telemetry'`) — staged to replace the loaded `wat-telemetry`
;; battery bridge, so no collision. A baked core source may declare under `:wat::` (stdlib bypasses
;; the reserved-prefix gate — RegistrationPrivilege::Stdlib in src/types.rs).
;;
;; Loads AFTER wat/core.wat (defrecord/defenum/defsurface/typealias + splice + Keyword/String/i64/
;; HashMap primitives). Depends additionally on :wat::core::Uuid (arc-207 runtime primitive, always
;; available).
;;
;; Arc 278 stone T1b.1 adds the `Journal` surface (write half) at the end of this file — it reuses
;; `:wat::query::{Constraint,Transient,Fatal}` (wat/query.wat) as its response payloads, so this
;; file's stdlib.rs manifest slot now ALSO depends on wat/query.wat and must load after it.

;; ─── Tags — the dimension map every scope carries (keyword → string). ────────────
(:wat::core::typealias :wat::telemetry'::Tags
  (:wat::core::HashMap :wat::core::keyword :wat::core::String))

;; ─── Numeric — a metric's value: an i64 count or an f64 gauge (fielded variants). ─
;; Variant names are :I64/:F64 (capitalized, per the sqlite Cell/Param exemplar): the
;; lowercase :i64/:f64 the design doc sketched collide with the RETIRED bare primitives
;; :i64/:f64 (arc-109) and are rejected as enum-variant names.
(:wat::core::defenum :wat::telemetry'::Numeric :wat::enum::Pure
  :I64 [val <- :wat::core::i64]
  :F64 [val <- :wat::core::f64])

;; ─── Unit — the unit a metric's value is measured in (bare variants). ────────────
(:wat::core::defenum :wat::telemetry'::Unit :wat::enum::Pure
  :Count
  :Nanos
  :Millis
  :Bytes
  :Percent)

;; ─── Level — a log record's severity (bare variants). ────────────────────────────
(:wat::core::defenum :wat::telemetry'::Level :wat::enum::Pure
  :Debug
  :Info
  :Warn
  :Error)

;; ─── Kind — which telemetry record a store partition holds (bare variants). ───────
;; Discriminates the two record shapes at the partition-key level (metrics and logs
;; are different shapes; the pk carries the kind so a namespace's metrics and logs
;; partition distinctly).
(:wat::core::defenum :wat::telemetry'::Kind :wat::enum::Pure
  :Metric
  :Log)

;; ─── PartitionKey — the store partition key (the `pk`): a tagged, PARSEABLE key. ──
;; Written via `:wat::edn::write` as `#wat.telemetry'/PartitionKey {:namespace … :kind …}`
;; — self-describing AND round-trippable (an EDN reader hydrates it back to this record),
;; unlike a `#`-delimited flat string which cannot be read back. Fields render in
;; declaration order, so the partition groups hierarchically (namespace, then kind).
(:wat::core::defrecord :wat::telemetry'::PartitionKey
  [namespace <- :wat::core::String
   kind      <- :wat::telemetry'::Kind])

;; ─── Scope — the EXACT surface every telemetry record satisfies (identity + when). ─
;; namespace (facility), uuid (correlation id), tags (dimensions), time-ns (event time).
;; Spliced into Metric/Log via `~@:wat::telemetry'::Scope`.
(:wat::core::defsurface :wat::telemetry'::Scope
  :nature :wat::core::Record
  :features [namespace <- :wat::core::String
             uuid      <- :wat::core::Uuid
             tags      <- :wat::telemetry'::Tags
             time-ns   <- :wat::core::i64])

;; ─── LogMessage — an OPEN surface (any record with a message shape satisfies it). ─
(:wat::core::defsurface :wat::telemetry'::LogMessage
  :nature :wat::core::Record
  :features [])

;; ─── Metric — a measurement. Splices Scope (4 fields), then 4 own. ───────────────
;; Ctor field order (splice-first, arc-293): namespace uuid tags time-ns  start-time-ns name value unit.
(:wat::core::defrecord :wat::telemetry'::Metric
  [~@:wat::telemetry'::Scope
   start-time-ns <- :wat::core::i64
   name          <- :wat::core::keyword
   value         <- :wat::telemetry'::Numeric
   unit          <- :wat::telemetry'::Unit])

;; ─── Log — a log event. Splices Scope (4 fields), then 3 own. ────────────────────
;; Ctor field order (splice-first, arc-293): namespace uuid tags time-ns  caller level message.
(:wat::core::defrecord :wat::telemetry'::Log
  [~@:wat::telemetry'::Scope
   caller  <- :wat::core::keyword
   level   <- :wat::telemetry'::Level
   message <- :wat::telemetry'::LogMessage])

;; ─── Journal — arc 278 stone T1b.1: the telemetry sink's S4c contract, write half. ─
;; A `:nature :wat::kernel::Peer'` surface — a dialed `Peer'<Journal::Op,Journal::Reply>` IS a
;; Journal intrinsically (arc 293 Path B), exactly the shape `:wat::query::Store` has
;; (wat/query.wat:101). Mirrors Store's `:messages`/`:features` split verbatim: per-op
;; `Journal::<Op>Request` records + `Journal::<Op>Response` `:wat::enum::Pure` enums in
;; `:messages`, kebab methods in `:features`. `journal'` (the satisfier, STONE T1b.2) holds a
;; `:wat::query::Store` peer and serializes Metric/Log -> StoredRow -> store/put; the WRITE
;; failures below are therefore the store's `put` failures, surfaced pass-through — NOT a
;; parallel telemetry error vocabulary (derive-is-the-wall). This is why `Journal` depends on
;; `:wat::query::` (Constraint/Transient/Fatal, wat/query.wat:78-80) and must load after it — see
;; this file's stdlib.rs manifest slot.
;;
;; WRITE half only (`write-metrics`/`write-logs`) — `query-metrics`/`query-logs` join the surface
;; at T2 (need `:wat::query::Query`/`Result` + the rete filter, absent today; see
;; docs/arc/2026/06/278-rules-engine/DESIGN-STONE-T1b1-journal-surface.md § Scope).
(:wat::core::defsurface :wat::telemetry'::Journal :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::telemetry'::Journal::WriteMetricsRequest
     [batch <- (:wat::core::Vector :wat::telemetry'::Metric)])
   (:wat::core::defenum :wat::telemetry'::Journal::WriteMetricsResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])

   (:wat::core::defrecord :wat::telemetry'::Journal::WriteLogsRequest
     [batch <- (:wat::core::Vector :wat::telemetry'::Log)])
   (:wat::core::defenum :wat::telemetry'::Journal::WriteLogsResponse :wat::enum::Pure
     :Success    []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])]
  :features
  [;; write a metrics batch (>=1, homogeneous) ATOMICALLY through the owned store.
   (write-metrics [self <- :wat::telemetry'::Journal  req <- :wat::telemetry'::Journal::WriteMetricsRequest]
     -> :wat::telemetry'::Journal::WriteMetricsResponse)

   ;; write a logs batch (>=1, homogeneous) ATOMICALLY through the owned store.
   (write-logs [self <- :wat::telemetry'::Journal  req <- :wat::telemetry'::Journal::WriteLogsRequest]
     -> :wat::telemetry'::Journal::WriteLogsResponse)])

;; ─── Span — arc 278 stone Span.1: the PRODUCER surface (a unit of work). ──────────
;; A short-lived `:nature :wat::kernel::Peer'` service the caller opens, works through, and closes.
;; `incr`/`timed` accumulate PURE state (counters + duration samples); `log` writes through the sink
;; NOW; `close` emits the accumulated counters + durations as Metrics to the sink (each counter -> 1
;; Metric; each duration name -> count + sum Metrics) — so CloseResponse passes through the sink's
;; write outcome (the shared :wat::query:: error vocab, derive-is-the-wall). `span'` (the satisfier,
;; stone Span.2) holds a `:wat::telemetry'::Journal` peer. Nesting is a call-site `open` with the same
;; sink (NOT a surface op). `timed` the OP (`Span/timed`) is distinct from the `timed` call-site widget
;; macro (`:wat::telemetry'::timed`) — FQDN disambiguates.
(:wat::core::defsurface :wat::telemetry'::Span :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :wat::telemetry'::Span::IncrRequest
     [name <- :wat::core::keyword])
   (:wat::core::defenum :wat::telemetry'::Span::IncrResponse :wat::enum::Pure :Ok [])

   (:wat::core::defrecord :wat::telemetry'::Span::TimedRequest
     [name <- :wat::core::keyword  nanos <- :wat::core::i64])
   (:wat::core::defenum :wat::telemetry'::Span::TimedResponse :wat::enum::Pure :Ok [])

   (:wat::core::defrecord :wat::telemetry'::Span::LogRequest
     [caller  <- :wat::core::keyword
      level   <- :wat::telemetry'::Level
      message <- :wat::telemetry'::LogMessage])
   (:wat::core::defenum :wat::telemetry'::Span::LogResponse :wat::enum::Pure :Ok [])

   (:wat::core::defrecord :wat::telemetry'::Span::CloseRequest [])
   (:wat::core::defenum :wat::telemetry'::Span::CloseResponse :wat::enum::Pure
     :Done       []
     :Constraint [err <- :wat::query::Constraint]
     :Transient  [err <- :wat::query::Transient]
     :Fatal      [err <- :wat::query::Fatal])]
  :features
  [;; increment a named counter by 1 — a PURE state transition (emitted on close).
   (incr [self <- :wat::telemetry'::Span  req <- :wat::telemetry'::Span::IncrRequest]
     -> :wat::telemetry'::Span::IncrResponse)
   ;; record a duration sample (nanos) under a name — PURE (the timing widget already measured).
   (timed [self <- :wat::telemetry'::Span  req <- :wat::telemetry'::Span::TimedRequest]
     -> :wat::telemetry'::Span::TimedResponse)
   ;; write a Log NOW through the sink, correlated by this span's scope.
   (log [self <- :wat::telemetry'::Span  req <- :wat::telemetry'::Span::LogRequest]
     -> :wat::telemetry'::Span::LogResponse)
   ;; close the unit of work: emit accumulated counters + durations as Metrics to the sink.
   (close [self <- :wat::telemetry'::Span  req <- :wat::telemetry'::Span::CloseRequest]
     -> :wat::telemetry'::Span::CloseResponse)])
