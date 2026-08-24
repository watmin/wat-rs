;; accumulator fence: a user fold whose body does IO. The message must name the :pure axis
;; and the offending head. Built-in `:wat::rete::acc::*` heads skip this fence.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::core::defn :wf::bad-fold
  [v <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::record? (:wat::io::IOReader/open-file "x")) v v))

(:wat::rete::defrule :wf::bad-acc
  :when
  [(:weather::Temperature (?c <- :celsius))
   (?s <- (:wf::bad-fold ?c) :from (:weather::Temperature (?c2 <- :celsius)))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::rete::compile rules)]
    (:wat::core::length (:wat::rete::Session/facts session))))
