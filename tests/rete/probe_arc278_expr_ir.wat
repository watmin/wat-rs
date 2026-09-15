;; #49 — compiled where. lower at rule-compile; exec == eval-test.

(:wat::core::defrecord :eir::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :eir::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :eir::q-Hit :params [] :when [(?fact <- :eir::Hit)])

(:wat::rete::defrule :eir::cool
  :when [(:eir::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))]
  :then [(:eir::Hit ?c)])

(:wat::core::defn :user::cmp-eval [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::rete::i64::< ?c 20))
    (:wat::core::PersistentMap "?c" 10)))

(:wat::core::defn :user::cmp-lower-ok [] -> :wat::core::nil
  (:wat::rete::lower (:wat::core::quote (:wat::rete::i64::< ?c 20))))

(:wat::core::defn :user::fire-cool [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::core::match (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:eir::cool))
                         (:wat::core::PersistentVector (:eir::q-Hit))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
                    s1 (:wat::core::match (:wat::rete::insert s0 (:eir::Temp :c 10)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    s2 (:wat::core::match (:wat::rete::insert s1 (:eir::Temp :c 30)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
                    fired (:wat::core::match (:wat::rete::fire-rules s2) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
    (:wat::core::length (:wat::rete::query fired (:eir::q-Hit)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::fire-cool)))
