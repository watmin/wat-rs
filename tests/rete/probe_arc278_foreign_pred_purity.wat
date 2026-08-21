;; tests/rete/probe_arc278_foreign_pred_purity.wat — co-located fixture for the sibling .rs,
;; slurped via call_beside(file!()). Arc 278 Stone sift-arena, Part A: the purity fence must
;; accept a FOREIGN-READER predicate (`:wat::edn::read-foreign` + `ForeignRecord/get`/`class`) —
;; the whole `:wat::edn::` namespace is pure data transforms (parse/serialize/navigate, no IO, no
;; entropy). Mirrors the accessor-purity idiom: each entry QUOTES the predicate under test and
;; hands it to the fence predicate — the quoted body is never evaluated.

(:wat::core::defn :user::foreign-pred-is-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
        (:wat::core::match
          (:wat::edn::read-foreign (:wat::telemetry::Log/message log))
          ((:wat::edn::ReadForeignOutcome::Value fr)
            (:wat::core::match (:wat::edn::ForeignRecord/get fr :severity)
              ((:wat::core::Some s) (:wat::core::= s "high"))
              (:wat::core::None false)))
          ((:wat::edn::ReadForeignOutcome::Malformed _) false))))))

(:wat::core::defn :user::foreign-pred-is-deterministic [] -> :wat::core::bool
  (:wat::rete::deterministic?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
        (:wat::core::match
          (:wat::edn::read-foreign (:wat::telemetry::Log/message log))
          ((:wat::edn::ReadForeignOutcome::Value fr)
            (:wat::core::match (:wat::edn::ForeignRecord/get fr :severity)
              ((:wat::core::Some s) (:wat::core::= s "high"))
              (:wat::core::None false)))
          ((:wat::edn::ReadForeignOutcome::Malformed _) false))))))

(:wat::core::defn :user::foreign-pred-is-total [] -> :wat::core::bool
  (:wat::rete::total?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::bool
        (:wat::core::match
          (:wat::edn::read-foreign (:wat::telemetry::Log/message log))
          ((:wat::edn::ReadForeignOutcome::Value fr)
            (:wat::core::match (:wat::edn::ForeignRecord/get fr :severity)
              ((:wat::core::Some s) (:wat::core::= s "high"))
              (:wat::core::None false)))
          ((:wat::edn::ReadForeignOutcome::Malformed _) false))))))

;; GUARD: the SAME predicate with an impure op (println) in the body must STILL be rejected — the
;; edn namespace fix is not a blanket-allow; the impure op's impurity must still propagate.
(:wat::core::defn :user::impure-foreign-pred-is-not-pure [] -> :wat::core::bool
  (:wat::rete::pure?
    (:wat::core::quote
      (:wat::core::fn [log <- :wat::telemetry::Log] -> :wat::core::nil
        (:wat::kernel::println
          (:wat::edn::ForeignRecord/get (:wat::edn::read-foreign (:wat::telemetry::Log/message log)) :severity))))))
