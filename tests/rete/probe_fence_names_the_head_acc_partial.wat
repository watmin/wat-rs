;; accumulator fence: user fold whose body is `i64::/` — must name :total, not :pure.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::core::defn :wf::partial-fold
  [v <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::/ v 1))

(:wat::rete::defrule :wf::bad-acc
  :when
  [(:weather::Temperature (?c :- :celsius))
   (?s :- (:wf::partial-fold ?c) :from (:weather::Temperature (?c2 :- :celsius)))]
  :then
  [(:wf::Gate :celsius ?c)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])]
    (:wat::core::length (:wat::rete::factbag::items (:wat::rete::Session/facts session)))))
