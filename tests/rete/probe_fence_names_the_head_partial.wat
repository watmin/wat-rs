;; tests/rete/probe_fence_names_the_head_partial.wat — partial-but-pure-and-det where world for the
;; probe_fence_names_the_head RED probe. `(:wat::core::i64::/ ?c 1)` is pure and deterministic but
;; NOT total (undefined at a zero divisor) — the compile fence must reject on the :total axis
;; specifically and name ":wat::core::i64::/" as the offending head. Mirrors
;; tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat's shape exactly, swapping the violating verb.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-gate
  :when
  [(:weather::Temperature (?c <- :celsius))
   (:wat::rete::where (:wat::core::i64::/ ?c 1))]
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
       fired   (:wat::core::match (:wat::rete::fire-rules$oracle session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
      (:wat::rete::query fired (:wf::q-Gate)))))
