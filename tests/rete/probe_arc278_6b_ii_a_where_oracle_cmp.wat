;; tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat — comparison-gate world for the where_oracle probe;
;; loaded via startup_from_file. Rule filters Temperature by (where (> ?c 0)).

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wg::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wg::cold-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::> ?c 0))]
  :then
  (:wat::rete::insert (:wg::Gate ?c)))

