;; wat-scripts/perf/grid/where-accum-group.wat — unbound grouping in a leading :from.
;; Twin of where-accum-group.clj.
;;   [?c <- (acc/count) :from [Temp (?loc <- :loc)]]
;; Temps at MCI and ORD are two groups, not one global count.
;; Empty world with a group key does not emit bag-wide 0.
;; Acc-first + Wind at ?loc: Clara defers the accum; Wind MCI and no temps → {?c 0, ?loc MCI}.

(:wat::core::defrecord :wag::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wag::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wag::Busy [loc <- :wat::core::String n <- :wat::core::i64])

(:wat::rete::defrule :wag::count-by-loc
  :when [(?n :- (:wat::rete::acc::count) :from (:wag::Temp (?loc :- :loc)))]
  :then [(:wag::Busy :loc ?loc :n ?n)])

(:wat::rete::defrule :wag::acc-first-wind
  :when [(?n :- (:wat::rete::acc::count) :from (:wag::Temp (?loc :- :loc)))
         (:wag::Wind (?loc :- :loc) (?w :- :kph)
           (:wat::rete::i64::> ?w 10))]
  :then [(:wag::Busy :loc ?loc :n ?n)])

(:wat::rete::defquery :wag::q-Busy
  :params []
  :when [(?fact :- :wag::Busy)])


(:wat::core::defn :wag::n-busy [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wag::q-Busy))))

(:wat::core::defn :wag::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat
        (:wat::string::concat " " name)
        (:wat::string::concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [by  (:wat::core::PersistentVector (:wag::count-by-loc))
                    af  (:wat::core::PersistentVector (:wag::acc-first-wind))]
    (:wag::line 1 "two-locs"
      (:wag::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wag::Temp :c 10 :loc "MCI")
            (:wag::Temp :c 20 :loc "MCI")
            (:wag::Temp :c 30 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wag::line 2 "empty-group"
      (:wag::n-busy (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wag::line 3 "one-loc"
      (:wag::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all by (:wat::core::PersistentVector (:wag::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wag::Temp :c 10 :loc "MCI")
            (:wag::Temp :c 20 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wag::line 4 "acc-first-wind-empty-temp"
      (:wag::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all af (:wat::core::PersistentVector (:wag::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wag::Wind :kph 20 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wag::line 5 "acc-first-two-winds"
      (:wag::n-busy
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all af (:wat::core::PersistentVector (:wag::q-Busy))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wag::Wind :kph 20 :loc "MCI")
            (:wag::Wind :kph 20 :loc "SFO")
            (:wag::Temp :c 40 :loc "SFO")
            (:wag::Temp :c 50 :loc "SFO")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))))
