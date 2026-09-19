;; experiri (vigilia cast, 2026-09-08) — `:then` value-operand position.
;; :wat::rete::core::match, BARE enum-variant arm spelling: (:probe::E::A true).
;; This is the exact program shape that `harness-experiri/README.md` §2 recorded as REFUSED
;; pre-cure (2026-08-30) and CURED 2026-09-02 by strike-match-arm-is-not-a-call. Re-driving to
;; confirm the cure holds against current HEAD, in the position that was never wired into the
;; live reachability.rs gate. hit=:probe::E::A -> true, miss=:probe::E::B -> false. Expect: 1.
(:wat::core::defenum :probe::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :probe::E])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String  ok <- :wat::core::bool])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (?v <- :v))]
  :then
  [(:probe::Out :k ?k :ok (:wat::rete::core::match ?v [:probe::E.A {} true] [:probe::E.B {} false]))])

(:wat::rete::defquery :probe::q
  :params []
  :when [(?fact <- :probe::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :probe)
       session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __ft} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
       session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v :probe::E.A)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __ilimit :used __iused :staged __icount} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v :probe::E.B)) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __ilimit :used __iused :staged __icount} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])
       fired   (:wat::core::match (:wat::rete::fire-rules session) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])]
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::core::if (:probe::Out/ok (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact"))
            (:wat::i64::+ acc 1)
            acc))
        0
        (:wat::rete::query fired (:probe::q))))))
