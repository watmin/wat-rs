;; wat-scripts/perf/grid/where-not-bound.wat — :not with a left-bound var (Clara
;; test-accum-result-in-negation). Empty-seed alpha cannot see `?v < ?m`; both
;; impls must re-match facts under the token. Mixed 50/40 → n=0; tied 50/50 → n=1.
;;
;; Twin of where-not-bound.clj.

(:wat::core::defn :wnb::row-count [] -> :wat::core::i64 2)

(:wat::core::defrecord :wnb::Station [loc <- :wat::core::String])
(:wat::core::defrecord :wnb::Reading [loc <- :wat::core::String v <- :wat::core::i64])
(:wat::core::defrecord :wnb::Busy    [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :wnb::max-not-below
  :when
  [(:wnb::Station (?loc :- :loc))
   (?m :- (:wat::rete::acc::max ?v) :from (:wnb::Reading (?loc :- :loc) (?v :- :v)))
   (:wat::rete::not (:wnb::Reading (?loc :- :loc) (?v :- :v)
                      (:wat::rete::i64::< ?v ?m)))]
  :then
  [(:wnb::Busy :loc ?loc :n ?m)])

(:wat::rete::defquery :wnb::q-Busy
  :params []
  :when [(?fact :- :wnb::Busy)])


(:wat::core::defn :wnb::fire [lo <- :wat::core::i64  hi <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all (:wat::core::PersistentVector (:wnb::max-not-below)) (:wat::core::PersistentVector (:wnb::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
      (:wnb::Station :loc "OSL")
      (:wnb::Reading :loc "OSL" :v lo)
      (:wnb::Reading :loc "OSL" :v hi)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")]))

(:wat::core::defn :wnb::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat
        (:wat::string::concat " " name)
        (:wat::string::concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wnb::line 1 "max-not-below-mixed"
    (:wat::core::length (:wat::rete::query (:wnb::fire 50 40) (:wnb::q-Busy))))
  (:wnb::line 2 "max-not-below-tied"
    (:wat::core::length (:wat::rete::query (:wnb::fire 50 50) (:wnb::q-Busy)))))
