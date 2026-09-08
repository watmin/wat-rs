;; experiri (vigilia cast, 2026-09-08) — `:then` value-operand position.
;; LAW A CONTROL, SECOND PROBE: `:wat::core::>` was refused in `:then`, but the diagnostic named
;; TOTALITY ("then expr is not total"), not purity/RetePrimitive-ness (Law A's stated mechanism in
;; `:when`: "the rete query language is composed from RETE primitives"). So: is `:then` actually
;; enforcing "rete primitives only", or merely "registered-total heads only" — a DIFFERENT fence
;; that could let a core head slip through if it happens to be on whatever total-registry `:then`
;; consults? Probe with `:wat::core::not` — a CORE (non-rete) head that is unambiguously total
;; (boolean negation is defined for every bool). If this FIRES, `:then` admits at least one
;; `:wat::core::` head Law A would refuse everywhere else — a bypass, not a gap.
(:wat::core::defrecord :probe::In  [k <- :wat::core::String  v <- :wat::core::bool])
(:wat::core::defrecord :probe::Out [k <- :wat::core::String  ok <- :wat::core::bool])

(:wat::rete::defrule :probe::rule
  :when
  [(:probe::In (?k <- :k) (?v <- :v))]
  :then
  [(:probe::Out :k ?k :ok (:wat::core::not ?v))])

(:wat::rete::defquery :probe::q
  :params []
  :when [(?fact <- :probe::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::let
      [rules   (:wat::rete::collect-rules :probe)
       session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:probe::q))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! "compile: the rule set may not terminate" :wat::core::None :wat::core::None)))
       session (:wat::core::match (:wat::rete::insert session (:probe::In :k "hit"  :v false)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
       session (:wat::core::match (:wat::rete::insert session (:probe::In :k "miss" :v true)) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
       fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
          (:wat::core::if (:probe::Out/ok (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?fact") "query: ?fact"))
            (:wat::core::i64::+ acc 1)
            acc))
        0
        (:wat::rete::query fired (:probe::q))))))
