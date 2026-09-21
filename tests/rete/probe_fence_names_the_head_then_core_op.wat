;; then-item fence: `(:wat::i64::> ?c 0)` is pure, det, AND total, but not a rete spelling.
;; Law A must be named, not :total.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :wf::Gate            [celsius <- :wat::core::i64])

(:wat::rete::defrule :wf::bad-then
  :when
  [(:weather::Temperature (?c :- :celsius))]
  :then
  [(:wat::i64::> ?c 0)])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :wf)
     session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])]
    (:wat::core::length (:wat::rete::factbag::items (:wat::rete::Session/facts session)))))
