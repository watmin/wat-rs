;; wat-scripts/perf/grid/where-exists.wat — leading AND mid-chain :exists.
;; Twin of where-exists.clj.
;;   [:exists [Wind ?loc]]                         — one Hit per distinct loc
;;   [:exists [Temp ?loc]] [:exists [Wind ?loc]]   — both must exist at loc
;;   [:or [:exists [Caw]] [:exists [Temp < 20]]]   — either presence
;;   [Loc ?loc] [:exists [Wind ?loc]]              — left seed; two winds → one Hit

(:wat::core::defrecord :wex::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wex::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :wex::Caw  [t <- :wat::core::i64 w <- :wat::core::i64])
(:wat::core::defrecord :wex::Loc  [loc <- :wat::core::String])
(:wat::core::defrecord :wex::At   [loc <- :wat::core::String])
(:wat::core::defrecord :wex::Hit  [k <- :wat::core::i64])

(:wat::rete::defrule :wex::lead-wind
  :when [(:wat::rete::exists (:wex::Wind (?loc :- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::both-exist
  :when [(:wat::rete::exists (:wex::Temp (?loc :- :loc)))
         (:wat::rete::exists (:wex::Wind (?loc :- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::or-exists
  :when [(:wat::rete::or
           (:wat::rete::exists (:wex::Caw (?t :- :t)))
           (:wat::rete::exists
             (:wex::Temp (?c :- :c)
               (:wat::rete::i64::< ?c 20))))]
  :then [(:wex::Hit :k 1)])

;; Mid-chain: Loc is the left token. Exists binds nothing; two Winds → one At.
(:wat::rete::defrule :wex::mid-wind
  :when [(:wex::Loc (?loc :- :loc))
         (:wat::rete::exists (:wex::Wind (?loc :- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defrule :wex::mid-both
  :when [(:wex::Loc (?loc :- :loc))
         (:wat::rete::exists (:wex::Temp (?loc :- :loc)))
         (:wat::rete::exists (:wex::Wind (?loc :- :loc)))]
  :then [(:wex::At :loc ?loc)])

(:wat::rete::defquery :wex::q-At
  :params []
  :when [(?fact :- :wex::At)])


(:wat::rete::defquery :wex::q-Hit
  :params []
  :when [(?fact :- :wex::Hit)])


(:wat::core::defn :wex::n-at [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wex::q-At))))

(:wat::core::defn :wex::n-hit [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::length (:wat::rete::query s (:wex::q-Hit))))

(:wat::core::defn :wex::line [row <- :wat::core::i64 name <- :wat::core::String n <- :wat::core::i64] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat
      (:wat::string::concat "row " (:wat::i64::to-string row))
      (:wat::string::concat
        (:wat::string::concat " " name)
        (:wat::string::concat " n=" (:wat::i64::to-string n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [lead (:wat::core::PersistentVector (:wex::lead-wind))
                    both (:wat::core::PersistentVector (:wex::both-exist))
                    ore  (:wat::core::PersistentVector (:wex::or-exists))
                    mid  (:wat::core::PersistentVector (:wex::mid-wind))
                    mboth (:wat::core::PersistentVector (:wex::mid-both))]
    (:wex::line 1 "lead-empty"
      (:wex::n-at (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 2 "lead-two-same"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 3 "lead-two-locs"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 4 "lead-retract"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::rete::retract
            (:wat::rete::retract
              (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all lead (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                (:wex::Wind :kph 50 :loc "MCI")
                (:wex::Wind :kph 60 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
              (:wex::Wind :kph 50 :loc "MCI"))
            (:wex::Wind :kph 60 :loc "MCI"))) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 5 "and-wind-only"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 6 "and-diff-locs"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 7 "and-both-mci"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 8 "and-two-cities"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all both (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Temp :c 70 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 9 "or-empty"
      (:wex::n-hit (:wat::core::match (:wat::rete::fire-rules (:wat::core::match (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 10 "or-caw"
      (:wex::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Caw :t 10 :w 10)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 11 "or-temp"
      (:wex::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Temp :c 10 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 12 "or-both"
      (:wex::n-hit
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all ore (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Caw :t 10 :w 10)
            (:wex::Temp :c 10 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 13 "mid-loc-only"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Loc :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 14 "mid-wind-only"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Wind :kph 50 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 15 "mid-two-winds"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Loc :loc "MCI")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "MCI")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 16 "mid-two-locs"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mid (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 17 "mid-both-one-city"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mboth (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))
    (:wex::line 18 "mid-both-two-cities"
      (:wex::n-at
        (:wat::core::match (:wat::rete::fire-rules
          (:wat::core::match (:wat::rete::insert (:wat::core::match (:wat::rete::compile-all mboth (:wat::core::PersistentVector (:wex::q-At) (:wex::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
            (:wex::Loc :loc "MCI")
            (:wex::Loc :loc "ORD")
            (:wex::Wind :kph 50 :loc "MCI")
            (:wex::Temp :c 60 :loc "MCI")
            (:wex::Wind :kph 60 :loc "ORD")
            (:wex::Temp :c 70 :loc "ORD")) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])))))
