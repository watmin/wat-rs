;; tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat — user-fn gate world for the where_oracle probe;
;; loaded via startup_from_file. Rule filters Temperature by (where (:test::big? ?c)).

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wb::Gate            [celsius <- :wat::core::i64])

(:wat::rete::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool (:wat::rete::core::i64::> n 100))

(:wat::rete::defrule :wb::big-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:test::big? ?c))]
  :then
  [(:wb::Gate :celsius ?c)])

(:wat::rete::defquery :wb::q-Gate
  :params []
  :when [(?fact <- :wb::Gate)])


;; 3 — a USER-fn predicate in the where works through the network: big?(150) → one Gate.
(:wat::core::defn :user::run-gate-c150 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wb)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wb::q-Gate)))
       session (:wat::rete::insert session (:weather::Temperature :celsius 150 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:wb::q-Gate)))))

;; 3b — the same user-fn predicate blocks below threshold: big?(50) → zero.
(:wat::core::defn :user::run-gate-c50 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wb)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wb::q-Gate)))
       session (:wat::rete::insert session (:weather::Temperature :celsius 50 :location "Oslo"))
       fired   (:wat::rete::fire-rules-spec session)]
      (:wat::rete::query fired (:wb::q-Gate)))))

