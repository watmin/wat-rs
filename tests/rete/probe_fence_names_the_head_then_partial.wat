;; then-item fence: `(:wat::core::i64::/ ?c 1)` is pure and deterministic but NOT total.
;; The message must name the :total axis and the offending head.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-then
  :when
  [(:weather::Temperature (?c <- :celsius))]
  :then
  [(:wat::core::i64::/ ?c 1)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
