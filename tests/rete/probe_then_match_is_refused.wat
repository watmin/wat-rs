;; NEGATIVE — an EXHAUSTIVE match in a :then. The fence refuses match outright
;; because its totality axis is head-level and cannot see arms (STOP-1).
(:wat::core::defenum :nm::K :wat::enum::Pure :Aa [] :Bb [])
(:wat::core::defrecord :nm::Box [label <- :wat::core::String])
(:wat::core::defrecord :nm::Src [k <- :nm::K])

(:wat::rete::defrule :nm::go
  :when [(:nm::Src (?k <- :k))]
  :then [(:nm::Box :label (:wat::rete::core::match ?k
           [:nm::K.Aa {} "aa"]
           [:nm::K.Bb {} "bb"]))])

(:wat::core::defn :user::run-compile [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :nm)
     session (:wat::core::match (:wat::rete::compile rules) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])]
    (:wat::core::length (:wat::rete::factbag::items (:wat::rete::Session/facts session)))))
