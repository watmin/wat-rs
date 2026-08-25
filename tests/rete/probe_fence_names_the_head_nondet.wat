;; tests/rete/probe_fence_names_the_head_nondet.wat — non-deterministic-but-PURE where world for the
;; probe_fence_names_the_head RED probe. `(:wat::uuid::v4)` does no IO (pure) but is random (NOT
;; deterministic) — the compile fence must reject on the :deterministic axis specifically and name
;; ":wat::uuid::v4" as the offending head. Mirrors
;; tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat's shape exactly, swapping the violating verb.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::record? (:wat::uuid::v4)))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::rete::defquery :wf::q-Gate
  :params []
  :when [(?fact <- :wf::Gate)])


(:wat::core::defn :user::run-gate-c5 [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :wf)
       session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wf::q-Gate)))
       session (:wat::rete::insert session (:weather::Temperature :celsius 5 :location "Oslo"))
       fired   (:wat::rete::fire-rules$oracle session)]
      (:wat::rete::query fired (:wf::q-Gate)))))
