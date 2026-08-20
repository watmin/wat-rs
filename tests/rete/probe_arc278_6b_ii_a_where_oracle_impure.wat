;; tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat — impure-where world for the where_oracle probe;
;; loaded via startup_from_file. Rule's where is impure (io) — the compile fence must reject it.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::record? (:wat::io::IOReader/open-file "x")))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::rete::defquery :wf::q-Gate
  :params []
  :when [(?fact <- :wf::Gate)])


;; 4 — the compile FENCE rejects an impure `where` (io): compiling the rule raises.
(:wat::core::defn :user::run-gate-c5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wf)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wf::q-Gate)))
       session (:wat::rete::insert session (:weather::Temperature :celsius 5 :location "Oslo"))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:wf::q-Gate)))))

