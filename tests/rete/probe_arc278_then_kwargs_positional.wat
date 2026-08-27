;; `:then` KWARGS IN A RUNTIME-BUILT RULE ARE READ POSITIONALLY — the reproduction.
;;
;; Two rules, identical except for the ORDER the kwargs are written in, both built as `Rule`
;; VALUES rather than declared with `defrule`. They must derive the same facts.
;;
;; Witness = sum over rows of (a * 1000 + b). Src facts are (0,7) (1,8) (2,9), so:
;;   correct   (a=x, b=y) -> 7 + 1008 + 2009 = 3024
;;   transposed(a=y, b=x) -> 7000 + 8001 + 9002 = 24003
;; A row COUNT is identical either way, which is why this is a value.
(:wat::core::defrecord :tk::Src [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :tk::Two [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::rete::defquery :tk::q :params [] :when [(:tk::Two (?a <- :a) (?b <- :b))])

(:wat::core::defn :tk::rule [rhs <- :wat::WatAST] -> :wat::rete::Rule
  (:wat::rete::Rule :name "r"
    :lhs (:wat::core::PersistentVector
           (:wat::core::quasiquote (:tk::Src (?x <- :x) (?y <- :y))))
    :rhs (:wat::core::PersistentVector rhs)))

(:wat::core::defn :tk::witness [rhs <- :wat::WatAST] -> :wat::core::i64
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64  p <- :wat::core::PersistentMap] -> :wat::core::i64
      (:wat::core::i64::+ acc
        (:wat::core::i64::+
          (:wat::core::i64::* (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?a") "a") 1000)
          (:wat::core::Option/expect (:wat::core::PersistentMap/get p "?b") "b"))))
    0
    (:wat::rete::query
      (:wat::rete::fire-rules
        (:wat::rete::insert-all
          (:wat::rete::compile-all
            (:wat::core::PersistentVector (:tk::rule rhs))
            (:wat::core::PersistentVector (:tk::q)))
          (:wat::core::PersistentVector
            (:tk::Src :x 0 :y 7) (:tk::Src :x 1 :y 8) (:tk::Src :x 2 :y 9))))
      (:tk::q))))

;; [declaration-order  reversed-order]
(:wat::core::defn :user::rows [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::mapv
    (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
    (:wat::core::PersistentVector
      (:tk::witness (:wat::core::quasiquote (:tk::Two :a ?x :b ?y)))
      (:tk::witness (:wat::core::quasiquote (:tk::Two :b ?y :a ?x))))))
