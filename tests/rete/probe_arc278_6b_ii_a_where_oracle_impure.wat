;; tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat — impure-where world for the where_oracle probe;
;; loaded via startup_from_file. Rule's where is impure (io) — the compile fence must reject it.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::record? (:wat::io::IOReader/open-file "x")))]
  :then
  (:wat::rete::insert (:wf::Gate ?c)))

