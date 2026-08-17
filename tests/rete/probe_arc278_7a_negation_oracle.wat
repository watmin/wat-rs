;; tests/rete/probe_arc278_7a_negation_oracle.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the alert::unattended rule for negation oracle tests.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :ops::Maintenance     [location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])

(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))
   (:wat::rete::not (:ops::Maintenance (?loc <- :location)))]
  :then
  [(:alert::Unattended :location ?loc)])

(:wat::rete::defquery :alert::q-Unattended
  :params []
  :when [(?fact <- :alert::Unattended)])


;; Fire the oracle after the given inserts and count derived Unattended facts.

;; 1 — `:not` PASSES when the negated fact is ABSENT: Temp(Oslo), no Maintenance → 1 Unattended.
(:wat::core::defn :user::unattended-count-absent [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

;; 2 — `:not` BLOCKS when the negated fact is PRESENT and MATCHES: Temp(Oslo) + Maintenance(Oslo) → 0.
(:wat::core::defn :user::unattended-count-present-matching [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

;; 3 — `:not` PASSES when a negated fact exists but at a DIFFERENT binding (the shared-var join-filter):
;; Temp(Oslo) + Maintenance(Bergen) → the Bergen maintenance does NOT match ?loc=Oslo → 1 Unattended.
(:wat::core::defn :user::unattended-count-present-different [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Bergen"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

