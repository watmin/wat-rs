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
;; available). No eval-deps beyond those.

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
