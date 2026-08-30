;; accumulator fence: user fold whose body uses `i64::>` — Law A, not :total.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::core::defn :wf::core-fold
  [v <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::> v 0) v v))

(:wat::rete::defrule :wf::bad-acc
  :when
  [(:weather::Temperature (?c <- :celsius))
   (?s <- (:wf::core-fold ?c) :from (:weather::Temperature (?c2 <- :celsius)))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::core::match (:wat::rete::compile rules) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __fact-type) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::Session/facts session))))
