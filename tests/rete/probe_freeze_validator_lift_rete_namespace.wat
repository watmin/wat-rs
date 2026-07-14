;; tests/rete/probe_freeze_validator_lift_rete_namespace.wat — arc 294 9a follow-on: the
;; `defrule` freeze wall was lifted off a hardcoded call in `build_env` step 7.8 into a
;; pluggable `FreezeValidator` extension point (`src/freeze/validator.rs`), drained via
;; `inventory::iter` (mirrors the `RestrictionEntry` drain in the same fn).
;;
;; This fixture carries the SAME bare-keyword `:celsius` corruption
;; `src/rete/validate.rs`'s own `corrupt_when_clause_is_a_located_error` unit test exercises
;; via the lower-level `build_env` call directly — this integration test instead runs the
;; corruption through the real top-level `startup_beside` entry point, proving the wall still
;; fires via the generic drain (not a hand-rolled call) and that the boxed error's `to_edn()`
;; still tags `#wat.rete/MalformedClause` (dynamic dispatch through `Box<dyn
;; FreezeValidatorError>` preserves the concrete validator's own namespace).
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])
(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature :celsius (?loc <- :location) :location (?c <- :celsius))]
  :then
  (:wat::rete::insert (:alert::Unattended :location ?loc)))
