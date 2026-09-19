;; Tier B step C battery — door: defclause_regs.
;; A user defclause on a user name. Asks: does any :wat::-owned body probe it?
(:wat::core::defclause :battery::twice
  ([n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* n 2)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::to-string (:battery::twice 21))))
