;; POSITIVE — cond with a terminal :else in a :then still works. C refuses match, not cond.
(:wat::core::defenum :cd::K :wat::enum::Pure :Aa [] :Bb [])
(:wat::core::defrecord :cd::Box [label <- :wat::core::String])
(:wat::core::defrecord :cd::Src [k <- :cd::K])

(:wat::rete::defrule :cd::go
  :when [(:cd::Src (?k :- :k))]
  :then [(:cd::Box :label (:wat::rete::core::cond
           ((:wat::rete::core::enum::= ?k (:cd::K.Bb {})) "bb")
           (:else "other")))])

(:wat::rete::defquery :cd::q-Box
  :params []
  :when [(:cd::Box (?label :- :label))])

(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::core::let
    [rules (:wat::rete::collect-rules :cd)
     s0    (:wat::core::match (:wat::rete::insert
             (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cd::q-Box))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
             (:cd::Src :k (:cd::K.Bb {}))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
     fired (:wat::core::match (:wat::rete::fire-rules s0) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
     hits  (:wat::rete::query fired (:cd::q-Box))]
    (:wat::core::Option/expect
      (:wat::map::get (:wat::core::first hits) "?label")
      "q-Box: ?label")))
