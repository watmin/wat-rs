;; accumulator fence: user fold whose body is `i64::/` — must name :total, not :pure.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::core::defn :wf::partial-fold
  [v <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::/ v 1))

(:wat::rete::defrule :wf::bad-acc
  :when
  [(:weather::Temperature (?c <- :celsius))
   (?s <- (:wf::partial-fold ?c) :from (:weather::Temperature (?c2 <- :celsius)))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
