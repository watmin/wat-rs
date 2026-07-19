;; tests/rete/probe_arc278_accessor_purity.wat — co-located fixture for the sibling .rs,
;; slurped via call_beside(file!()). The 6a purity fence must read a generated field ACCESSOR's
;; DECLARED purity (the type's Nature / enum :wat::enum::* marker) the same way it already reads a
;; constructor's. A Record accessor (Log/level) is pure ∧ deterministic; an effectful body must
;; still be rejected. Each entry QUOTES the predicate under test and hands it to the fence
;; predicate — the quoted body is never evaluated (free/typed refs inside it are not checked).

(:wat::core::defn :user::log-accessor-is-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
        (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))))

(:wat::core::defn :user::log-accessor-is-deterministic [] -> :wat::core::bool
  (:wat::rete::deterministic?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
        (:wat::core::= (:wat::telemetry::Log/level log) :wat::telemetry::Level::Error)))))

;; GUARD: an effectful body (println) must STILL be rejected — the accessor fix must not
;; blanket-allow; the impurity of :wat::kernel::println must propagate through the fn-literal.
(:wat::core::defn :user::impure-accessor-body-is-not-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::nil
        (:wat::kernel::println (:wat::telemetry::Log/level log))))))
