;; tests/rete/probe_arc278_7b_negation_native_differential.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the alert::unattended rule for the native/oracle differential.

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


;; Fire via `fire` after the given inserts; count derived Unattended facts. Six combos:
;; {native, oracle} x {absent, present-matching, present-different}.

(:wat::core::defn :user::native-absent [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

(:wat::core::defn :user::oracle-absent [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

(:wat::core::defn :user::native-present-matching [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Oslo"))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

(:wat::core::defn :user::oracle-present-matching [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

(:wat::core::defn :user::native-present-different [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Bergen"))
       fired   (:wat::rete::fire-rules session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

(:wat::core::defn :user::oracle-present-different [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :alert)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:alert::q-Unattended)))
       session (:wat::rete::insert session (:weather::Temperature :celsius -5 :location "Oslo"))
       session (:wat::rete::insert session (:ops::Maintenance :location "Bergen"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:alert::q-Unattended)))))

