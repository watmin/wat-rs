;; `:then` KWARGS IN A RUNTIME-BUILT RULE ARE READ POSITIONALLY — the reproduction.
;;
;; Two rules, identical except for the ORDER the kwargs are written in, both built as `Rule`
;; VALUES rather than declared with `defrule`. They must derive the same facts.
;;
;; Witness = sum over rows of (a * 1000 + b). Src facts are (0,7) (1,8) (2,9), so:
;;   correct   (a=x, b=y) -> 7 + 1008 + 2009 = 3024
;;   transposed(a=y, b=x) -> 7000 + 8001 + 9002 = 24003
;; A row COUNT is identical either way, which is why this is a value.
(:wat::core::defrecord :tk::Src [x <- wat.type/i64  y <- wat.type/i64])
(:wat::core::defrecord :tk::Two [a <- wat.type/i64  b <- wat.type/i64])

(:wat::rete::defquery :tk::q :params [] :when [(:tk::Two (?a :- :a) (?b :- :b))])

(:wat::core::defn :tk::rule [rhs <- wat.type/AST] -> :wat::rete::Rule
  (:wat::rete::Rule :name "r"
    :lhs (wat.type/PersistentVector :- [wat.type/AST]
           (:wat::core::quasiquote (:tk::Src (?x :- :x) (?y :- :y))))
    :rhs (wat.type/PersistentVector :- [wat.type/AST] rhs)))

(:wat::core::defn :tk::witness [rhs <- wat.type/AST] -> wat.type/i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- wat.type/i64  p <- wat.type/PersistentMap] -> wat.type/i64
      (:wat::i64::+ acc
        (:wat::i64::+
          (:wat::i64::* (:wat::core::Option/expect (:wat::core::get p "?a") "a") 1000)
          (:wat::core::Option/expect (:wat::core::get p "?b") "b"))))
    0
    (:wat::rete::query
      (:wat::core::match (:wat::rete::fire-rules
        (:wat::core::match (:wat::rete::insert-all
          (:wat::core::match (:wat::rete::compile-all
            (wat.type/PersistentVector :- [:wat::rete::Rule] (:tk::rule rhs))
            (wat.type/PersistentVector :- [:wat::rete::Query] (:tk::q))) [:wat::rete::CompileOutcome.Compiled {:session __session} __session] [:wat::rete::CompileOutcome.MayNotTerminate {:rule __rule :fact-type __fact-type} (:wat::kernel::assertion-failed! :message "compile: the rule set may not terminate")])
          (wat.type/PersistentVector :- [:tk::Src]
            (:tk::Src :x 0 :y 7) (:tk::Src :x 1 :y 8) (:tk::Src :x 2 :y 9))) [:wat::rete::InsertOutcome.Inserted {:session __staged} __staged] [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __limit :used __used :staged __count} (:wat::kernel::assertion-failed! :message "insert: session memory ceiling exceeded while staging")])) [:wat::rete::FireOutcome.Fired {:value __fired} __fired] [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __limit :used __used :rounds __rounds} (:wat::kernel::assertion-failed! :message "fire-rules: session memory ceiling exceeded")] [:wat::rete::FireOutcome.RoundCapExceeded {:cap __cap :still-deriving __still} (:wat::kernel::assertion-failed! :message "fire-rules: fixpoint round cap exceeded")])
      (:tk::q))))

;; [declaration-order  reversed-order]
(:wat::core::defn :user::rows [] -> (wat.type/Vector :- [wat.type/i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64 n)
    (wat.type/PersistentVector :- [wat.type/i64]
      (:tk::witness (:wat::core::quasiquote (:tk::Two :a ?x :b ?y)))
      (:tk::witness (:wat::core::quasiquote (:tk::Two :b ?y :a ?x))))))
