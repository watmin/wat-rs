;; Disconfirming probe — is `bigint` a genuine DIFFERENTIAL partner for `i64`?
;; Two independent implementations of integer arithmetic; the law compares them to each
;; other, so nothing here is an oracle anyone invented.
(:wat::core::defrecord :probe::Pair [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::core::defn :probe::law-add [p <- :probe::Pair] -> :wat::core::bool
  (:wat::core::let [a (:probe::Pair/a p)  b (:probe::Pair/b p)]
    (:wat::core::= (:wat::core::i64::to-bigint (:wat::core::i64::+ a b))
                   (:wat::core::bigint::+ (:wat::core::i64::to-bigint a)
                                          (:wat::core::i64::to-bigint b)))))

(:wat::core::defn :probe::law-mul [p <- :probe::Pair] -> :wat::core::bool
  (:wat::core::let [a (:probe::Pair/a p)  b (:probe::Pair/b p)]
    (:wat::core::= (:wat::core::i64::to-bigint (:wat::core::i64::* a b))
                   (:wat::core::bigint::* (:wat::core::i64::to-bigint a)
                                          (:wat::core::i64::to-bigint b)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [g (:wat::gen::record :probe::Pair (:wat::gen::ints -20 21) (:wat::gen::ints -20 21))]
    (:wat::kernel::println
      (:wat::edn::write (:wat::gen::check g :probe::law-add)))
    (:wat::kernel::println
      (:wat::edn::write (:wat::gen::check g :probe::law-mul)))))
