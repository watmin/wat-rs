;; tests/rete/probe_arc278_sieve_pred.wat — co-located fixture for the sibling .rs, slurped via
;; call_beside(file!()). Arc 278 Stone 2: `:wat::query::sieve-pred` must capture a real
;; `(fn [log <- :T] -> :bool …)`, `ast->source`-print it verbatim into a `Sieve::Predicate` field,
;; and round-trip through `read-string` back to the same form. Sieve currently has no field
;; accessor (enums mint constructors only, never accessors — see register_enum_methods); `match`
;; destructures the `pred` field, mirroring every other Sieve/Store-response consumer in the tree.

;; verbatim ::-source: the captured pred string must contain "::" (organic UX — a real captured
;; fn-form, never hand-typed EDN).
(:wat::core::defn :user::sieve-pred-contains-double-colon [] -> :wat::core::bool
  (:wat::core::let
    [sieve (:wat::query::sieve-pred
             (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
               (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))
     pred-src (:wat::core::match sieve 
                ((:wat::query::Sieve::Predicate pred) pred))]
    (:wat::string::contains? pred-src "::")))

;; round-trip: read-string(pred) reproduces the SAME fn-form the user wrote (compared against an
;; independently-quoted copy of the identical form, per the ast-to-source probe's pattern).
(:wat::core::defn :user::sieve-pred-round-trips [] -> :wat::core::bool
  (:wat::core::let
    [fn-form (:wat::core::quote
               (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
                 (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))
     sieve   (:wat::query::sieve-pred
               (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
                 (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))
     pred-src (:wat::core::match sieve 
                ((:wat::query::Sieve::Predicate pred) pred))
     rebuilt (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string pred-src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))]
    (:wat::core::= fn-form rebuilt)))
