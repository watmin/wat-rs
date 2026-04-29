;; :wat::measure::* type aliases — the shared shapes used by
;; WorkUnit, Event::Metric, Event::Log, and any consumer that
;; constructs / reads measurement-event tags.
;;
;; Lives here (rather than inline next to WorkUnit) because the
;; same shapes appear on multiple surfaces — extracting them
;; clarifies dependency direction (WorkUnit / Event types USE
;; these; they aren't OWNED by either). Per the user's direction
;; mid-arc-091-slice-4: "your typedefs should be a wat/ file that
;; you load! in the prelude".
;;
;; Registered ahead of WorkUnit.wat in wat-measure's
;; `wat_sources()` so every later file in the crate sees the
;; aliases at parse time. Tests inherit them through the same
;; registration without needing explicit load! in the prelude.


;; A single tag's K,V shape — the form `:wat::core::HashMap`'s
;; constructor takes as its first argument. With slice-4's
;; substrate fix (typealias expansion at the constructor's
;; first-arg check, mirroring the "aliases resolve structurally
;; at call sites" rule), `(:wat::core::HashMap :wat::measure::Tag
;; ...)` works exactly as if the literal `:(K,V)` were written.
(:wat::core::typealias :wat::measure::Tag
  :(wat::holon::HolonAST,wat::holon::HolonAST))


;; The wu's tag map shape — arbitrary HolonAST→HolonAST pairs
;; that ride on every emitted Event row as a queryable EDN map.
;; Aliased per arc 077's "nested generics get a typealias"
;; convention; resolves structurally to
;; `:HashMap<wat::holon::HolonAST,wat::holon::HolonAST>` at every
;; declaration site.
(:wat::core::typealias :wat::measure::Tags
  :HashMap<wat::holon::HolonAST,wat::holon::HolonAST>)


;; The bundled Service<Event,_> handles the consumer's
;; `WorkUnit/scope` body needs to ship Events at scope-close.
;; The substrate-defined `:wat::measure::Event` enum is what the
;; consumer's Service is parameterized on; SinkHandles wraps the
;; (req-tx, ack-tx, ack-rx) triple so scope's signature stays
;; flat. Per arc 077's "nested-generic alias" convention.
(:wat::core::typealias :wat::measure::SinkHandles
  :(wat::telemetry::Service::ReqTx<wat::measure::Event>,wat::telemetry::Service::AckTx,wat::telemetry::Service::AckRx))
