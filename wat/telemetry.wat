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
(:wat::core::typealias :wat::telemetry::Tags
  (:wat::core::HashMap :- [:wat::core::keyword :wat::core::String]))

;; ─── Samples — a span's duration samples (nanos) under one name. A bare keyword alias so it
;; can name a HashMap value type + a `match ->` annotation (compound types can't sit there). ─
(:wat::core::typealias :wat::telemetry::Samples
  (:wat::core::Vector :- [:wat::core::i64]))

;; ─── Numeric — a metric's value: an i64 count or an f64 gauge (fielded variants). ─
;; Variant names are :I64/:F64 (capitalized, per the sqlite Cell/Param exemplar): the
;; lowercase :i64/:f64 the design doc sketched collide with the RETIRED bare primitives
;; :i64/:f64 (arc-109) and are rejected as enum-variant names.
(:wat::core::defenum :wat::telemetry::Numeric :wat::enum::Pure
  :I64 [val <- :wat::core::i64]
  :F64 [val <- :wat::core::f64])

;; ─── Unit — the unit a metric's value is measured in (bare variants). ────────────
(:wat::core::defenum :wat::telemetry::Unit :wat::enum::Pure
  :Count
  :Nanos
  :Millis
  :Bytes
  :Percent)

;; ─── Level — a log record's severity (bare variants). ────────────────────────────
(:wat::core::defenum :wat::telemetry::Level :wat::enum::Pure
  :Debug
  :Info
  :Warn
  :Error)

;; ─── Kind — which telemetry record a store partition holds (bare variants). ───────
;; Discriminates the two record shapes at the partition-key level (metrics and logs
;; are different shapes; the pk carries the kind so a namespace's metrics and logs
;; partition distinctly).
(:wat::core::defenum :wat::telemetry::Kind :wat::enum::Pure
  :Metric
  :Log)

;; ─── PartitionKey — the store partition key (the `pk`): a tagged, PARSEABLE key. ──
;; Written via `:wat::edn::write` as `#wat.telemetry'/PartitionKey {:namespace … :kind …}`
;; — self-describing AND round-trippable (an EDN reader hydrates it back to this record),
;; unlike a `#`-delimited flat string which cannot be read back. Fields render in
;; declaration order, so the partition groups hierarchically (namespace, then kind).
(:wat::core::defrecord :wat::telemetry::PartitionKey
  [namespace <- :wat::core::String
   kind      <- :wat::telemetry::Kind])

;; ─── Scope — the EXACT surface every telemetry record satisfies (identity + when). ─
;; namespace (facility), uuid (correlation id), tags (dimensions), time-ns (event time).
;; Spliced into Metric/Log via `~@:wat::telemetry'::Scope`.
(:wat::core::defsurface :wat::telemetry::Scope
  :nature :wat::core::Record
  :features [namespace <- :wat::core::String
             uuid      <- :wat::core::Uuid
             tags      <- :wat::telemetry::Tags
             time-ns   <- :wat::core::i64])

;; ─── Metric — a measurement. Splices Scope (4 fields), then 4 own. ───────────────
;; Ctor field order (splice-first, arc-293): namespace uuid tags time-ns  start-time-ns name value unit.
(:wat::core::defrecord :wat::telemetry::Metric
  [~@:wat::telemetry::Scope
   start-time-ns <- :wat::core::i64
   name          <- :wat::core::keyword
   value         <- :wat::telemetry::Numeric
   unit          <- :wat::telemetry::Unit])

;; ─── Log — a log event. Splices Scope (4 fields), then 3 own. ────────────────────
;; Ctor field order (splice-first, arc-293): namespace uuid tags time-ns  emitted-from level message.
(:wat::core::defrecord :wat::telemetry::Log
  [~@:wat::telemetry::Scope
   emitted-from  <- :wat::kernel::Frame
   level   <- :wat::telemetry::Level
   ;; message is OPAQUE (arc 278 Stone B): EDN text the producer `edn::write`s at the call site;
   ;; the sink stores/returns it verbatim and never decodes (no `UnknownTag` across a fork).
   message <- :wat::core::String])

;; ─── Journal — arc 278 stone T1b.1: the telemetry sink's S4c contract, write half. ─
;; A `:nature :wat::kernel::Peer'` surface — a dialed `(Peer' :- [Journal::Op Journal::Reply])` IS a
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
;; The minimal-CloudWatch contract: write + query, for metrics + logs. `write-*` persist a batch;
;; `query-*` read a namespace back over a time window (a filtered store scan, hydrating the rows to
;; Metric/Log — NO rete: rete is a CONSUMER that instruments itself and queries back, not the engine).
(:wat::core::defsurface :wat::telemetry::Journal :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::telemetry::Journal::WriteMetricsRequest
     [batch <- (:wat::core::Vector :- [:wat::telemetry::Metric])])
   (:wat::core::defenum :wat::telemetry::Journal::WriteMetricsResponse :wat::enum::Pure
     :Success        []
     :Constraint     [err <- :wat::query::Constraint]
     :Transient      [err <- :wat::query::Transient]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Journal::WriteLogsRequest
     [batch <- (:wat::core::Vector :- [:wat::telemetry::Log])])
   (:wat::core::defenum :wat::telemetry::Journal::WriteLogsResponse :wat::enum::Pure
     :Success        []
     :Constraint     [err <- :wat::query::Constraint]
     :Transient      [err <- :wat::query::Transient]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   ;; ── query (CloudWatch read side): a namespace + time window [lo,hi] in epoch nanos, paged. ──
   (:wat::core::defrecord :wat::telemetry::Journal::QueryMetricsRequest
     [namespace <- :wat::core::String
      time-lo   <- :wat::core::i64
      time-hi   <- :wat::core::i64
      limit     <- :wat::core::i64
      cursor    <- (:wat::core::Option :- [:wat::core::String])])
   ;; scan yields Success/Transient/Fatal only (a read can't constraint-fail) — mirror that.
   (:wat::core::defenum :wat::telemetry::Journal::QueryMetricsResponse :wat::enum::Pure
     :Success   [metrics <- (:wat::core::Vector :- [:wat::telemetry::Metric])
                 cursor  <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Journal::QueryLogsRequest
     [namespace <- :wat::core::String
      time-lo   <- :wat::core::i64
      time-hi   <- :wat::core::i64
      limit     <- :wat::core::i64
      cursor    <- (:wat::core::Option :- [:wat::core::String])])
   (:wat::core::defenum :wat::telemetry::Journal::QueryLogsResponse :wat::enum::Pure
     :Success   [logs   <- (:wat::core::Vector :- [:wat::telemetry::Log])
                 cursor <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   ;; ── sift (arc 278 Stone 2 — server-side filtering, DESIGN-sift-server-side-filter.md): the
   ;; same namespace + time-window page as query-*, PLUS a `Sieve` (the pure filter spec — this
   ;; stone only ships `Sieve::Predicate`). The op compiles the predicate ONCE, applies it per
   ;; row, and returns only survivors; an impure/non-deterministic predicate is REJECTED —
   ;; `::Fatal` with a Fault, never a silent pass. ──
   (:wat::core::defrecord :wat::telemetry::Journal::SiftLogsRequest
     [namespace <- :wat::core::String
      time-lo   <- :wat::core::i64
      time-hi   <- :wat::core::i64
      limit     <- :wat::core::i64
      cursor    <- (:wat::core::Option :- [:wat::core::String])
      sieve     <- :wat::query::Sieve])
   (:wat::core::defenum :wat::telemetry::Journal::SiftLogsResponse :wat::enum::Pure
     :Success   [logs   <- (:wat::core::Vector :- [:wat::telemetry::Log])
                 cursor <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Journal::SiftMetricsRequest
     [namespace <- :wat::core::String
      time-lo   <- :wat::core::i64
      time-hi   <- :wat::core::i64
      limit     <- :wat::core::i64
      cursor    <- (:wat::core::Option :- [:wat::core::String])
      sieve     <- :wat::query::Sieve])
   (:wat::core::defenum :wat::telemetry::Journal::SiftMetricsResponse :wat::enum::Pure
     :Success   [metrics <- (:wat::core::Vector :- [:wat::telemetry::Metric])
                 cursor  <- (:wat::core::Option :- [:wat::core::String])]
     :Transient [err <- :wat::query::Transient]
     :Fatal     [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [;; write a metrics batch (>=1, homogeneous) ATOMICALLY through the owned store.
   (write-metrics [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::WriteMetricsRequest]
     -> :wat::telemetry::Journal::WriteMetricsResponse :max-request-bytes 10485760)

   ;; write a logs batch (>=1, homogeneous) ATOMICALLY through the owned store.
   (write-logs [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::WriteLogsRequest]
     -> :wat::telemetry::Journal::WriteLogsResponse :max-request-bytes 10485760)

   ;; query metrics in a namespace over [time-lo, time-hi] — scan + hydrate, paged by cursor.
   (query-metrics [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::QueryMetricsRequest]
     -> :wat::telemetry::Journal::QueryMetricsResponse :max-request-bytes 524288)

   ;; query logs in a namespace over [time-lo, time-hi] — scan + hydrate, paged by cursor.
   (query-logs [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::QueryLogsRequest]
     -> :wat::telemetry::Journal::QueryLogsResponse :max-request-bytes 524288)

   ;; sift logs — query-logs + server-side filtering (Sieve compiled once, applied per row).
   (sift-logs [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::SiftLogsRequest]
     -> :wat::telemetry::Journal::SiftLogsResponse :max-request-bytes 524288)

   ;; sift metrics — the mechanical twin, over the Metric partition.
   (sift-metrics [self <- :wat::telemetry::Journal  req <- :wat::telemetry::Journal::SiftMetricsRequest]
     -> :wat::telemetry::Journal::SiftMetricsResponse :max-request-bytes 524288)])

;; ─── Span — arc 278 stone Span.1: the PRODUCER surface (a unit of work). ──────────
;; A short-lived `:nature :wat::kernel::Peer'` service the caller opens, works through, and closes.
;; `incr`/`timed` accumulate PURE state (counters + duration samples); `log` writes through the sink
;; NOW; `close` emits the accumulated counters + durations as Metrics to the sink (each counter -> 1
;; Metric; each duration name -> count + sum Metrics) — so CloseResponse passes through the sink's
;; write outcome (the shared :wat::query:: error vocab, derive-is-the-wall). `span'` (the satisfier,
;; stone Span.2) holds a `:wat::telemetry'::Journal` peer. Nesting is a call-site `open` with the same
;; sink (NOT a surface op). `timed` the OP (`Span/timed`) is distinct from the `timed` call-site widget
;; macro (`:wat::telemetry'::timed`) — FQDN disambiguates.
(:wat::core::defsurface :wat::telemetry::Span :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat::telemetry::Span::IncrRequest
     [name <- :wat::core::keyword])
   (:wat::core::defenum :wat::telemetry::Span::IncrResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Span::TimedRequest
     [name <- :wat::core::keyword  nanos <- :wat::core::i64])
   (:wat::core::defenum :wat::telemetry::Span::TimedResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Span::LogRequest
     [emitted-from  <- :wat::kernel::Frame
      level   <- :wat::telemetry::Level
      ;; message OPAQUE (arc 278 Stone B): the `Span/log` caller `edn::write`s its record here, so
      ;; a forked `span'` never hits `UnknownTag` on a user type either — opaque before both wires.
      message <- :wat::core::String])
   (:wat::core::defenum :wat::telemetry::Span::LogResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])

   (:wat::core::defrecord :wat::telemetry::Span::CloseRequest [])
   (:wat::core::defenum :wat::telemetry::Span::CloseResponse :wat::enum::Pure
     :Done           []
     :Constraint     [err <- :wat::query::Constraint]
     :Transient      [err <- :wat::query::Transient]
     :Fatal          [err <- :wat::query::Fatal]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [;; increment a named counter by 1 — a PURE state transition (emitted on close).
   (incr [self <- :wat::telemetry::Span  req <- :wat::telemetry::Span::IncrRequest]
     -> :wat::telemetry::Span::IncrResponse :max-request-bytes 524288)
   ;; record a duration sample (nanos) under a name — PURE (the timing widget already measured).
   (timed [self <- :wat::telemetry::Span  req <- :wat::telemetry::Span::TimedRequest]
     -> :wat::telemetry::Span::TimedResponse :max-request-bytes 524288)
   ;; write a Log NOW through the sink, correlated by this span's scope.
   (log [self <- :wat::telemetry::Span  req <- :wat::telemetry::Span::LogRequest]
     -> :wat::telemetry::Span::LogResponse :max-request-bytes 524288)
   ;; close the unit of work: emit accumulated counters + durations as Metrics to the sink.
   (close [self <- :wat::telemetry::Span  req <- :wat::telemetry::Span::CloseRequest]
     -> :wat::telemetry::Span::CloseResponse :max-request-bytes 524288)])

;; ─── framing-floor-of — arc 278 capacity stone 1: the RUNTIME adaptive framing-floor derive. ──
;; DESIGN-telemetry-caller-and-capacity.md §3. Reflects a record type's fields at RUNTIME
;; (field-names-of/field-types-of — compile-time/macro-expand reflection of a baked record is
;; DEAD, proven; runtime resolves for both stdlib and user records) and sums the byte cost of:
;;   1. FIXED-VALUE costs — per field, classify its type-node's `ast-name` string against the
;;      "explicitly-defined-known-size" set (i64/f64/Uuid/bool, max EDN-text bytes). Everything
;;      else (String, Tags/maps, records, Frame, enums) is VARIABLE and contributes 0 here — an
;;      under-count is a SAFE conservative floor, never an over-count (the runtime remainder is
;;      the exact per-caller gate; enum-by-reflection via a future `variants-of` is the deferred
;;      refinement that moves enums from variable into fixed, sized to their longest variant).
;;   2. Field-name KEY costs — every field name is a wire key; its serialized byte cost is
;;      `string::length` of the field keyword's text (ASCII → char-length = byte-length; a UTF-8
;;      byte-length prim is a deferred refinement for non-ASCII keys).
;;   3. TAG cost — the record's own EDN tag bytes, approximated via `string::length` of the type
;;      keyword's own text (the fqdn passed in by the caller IS the tag written on the wire).
;; This is the whole point: re-run this on ANY type keyword and the floor RE-DERIVES from the
;; LIVE field set — a field added/removed/retyped tomorrow needs no hand edits here.
(:wat::core::defn :wat::telemetry::framing-floor-of [ty <- :wat::core::keyword] -> :wat::core::i64
  (:wat::core::let
    [tag-cost   (:wat::string::length (:wat::keyword::to-string ty))
     fixed-cost (:wat::core::foldl
                  (:wat::core::fn [acc <- :wat::core::i64  t <- :wat::WatAST] -> :wat::core::i64
                    (:wat::i64::+ acc
                      (:wat::core::cond
                        ((:wat::core::= (:wat::core::ast-name t) "wat.type/i64")  20)
                        ((:wat::core::= (:wat::core::ast-name t) "wat.type/f64")  24)
                        ((:wat::core::= (:wat::core::ast-name t) "wat.type/Uuid") 36)
                        ((:wat::core::= (:wat::core::ast-name t) "wat.type/bool")  5)
                        (:else 0))))
                  0 (:wat::runtime::field-types-of ty))
     key-cost   (:wat::core::foldl
                  (:wat::core::fn [acc <- :wat::core::i64  k <- :wat::core::keyword] -> :wat::core::i64
                    (:wat::i64::+ acc (:wat::string::length (:wat::keyword::to-string k))))
                  0 (:wat::runtime::field-names-of ty))]
    (:wat::i64::+ tag-cost (:wat::i64::+ fixed-cost key-cost))))

;; ─── LOG-MSG-CAPACITY — the derived, zero-waste-ish log message byte budget. ──────
;; BUDGET is the named server read ceiling (10 MiB — matches the `journal'`/`mem-store`
;; `:max-request-bytes 10485760` bulk-service declaration, DESIGN-service-io-budgets.md:41;
;; `:wat::spawn::DEFAULT-MAX-MESSAGE-BYTES`-style named constant, never a bare magic number).
;; LOG-MSG-CAPACITY is the ADVISORY remainder after the adaptive framing floor of `Log` itself —
;; re-derives automatically whenever `Log`'s field set changes (a field added tomorrow shrinks
;; this without a hand edit). This is a conservative HINT; the exact per-caller gate is the
;; runtime remainder against the actually-filled-in required params (§3, deferred wiring).
(:wat::core::def :wat::telemetry::LOG-JOURNAL-BUDGET-BYTES 10485760)
(:wat::core::def :wat::telemetry::LOG-MSG-CAPACITY
  (:wat::i64::- :wat::telemetry::LOG-JOURNAL-BUDGET-BYTES
    (:wat::telemetry::framing-floor-of :wat::telemetry::Log)))
