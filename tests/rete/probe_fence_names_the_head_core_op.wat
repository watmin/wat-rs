;; tests/rete/probe_fence_names_the_head_core_op.wat — total-but-not-rete where world for the
;; probe_fence_names_the_head RED probe. `(:wat::i64::> ?c 0)` is pure, deterministic, AND
;; total, but it is a core spelling — Law A (rete-primitive) must be named, not :total.
;; Mirrors tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat's shape exactly, swapping
;; the violating verb. A regression that reports this as "is not total" stays red.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::i64::> ?c 0))]
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
