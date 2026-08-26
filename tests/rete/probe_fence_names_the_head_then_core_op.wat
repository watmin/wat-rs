;; then-item fence: `(:wat::core::i64::> ?c 0)` is pure, det, AND total, but a core spelling.
;; Law A must be named, not :total.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-then
  :when
  [(:weather::Temperature (?c <- :celsius))]
  :then
  [(:wat::i64::> ?c 0)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
