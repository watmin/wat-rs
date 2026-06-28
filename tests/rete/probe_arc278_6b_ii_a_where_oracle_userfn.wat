;; tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat — user-fn gate world for the where_oracle probe;
;; loaded via startup_from_file. Rule filters Temperature by (where (:test::big? ?c)).

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wb::Gate            [celsius <- :wat::core::i64])

(:wat::core::defn :test::big? [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 100))

(:wat::rete::defrule :wb::big-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:test::big? ?c))]
  :then
  (:wat::rete::insert (:wb::Gate ?c)))

