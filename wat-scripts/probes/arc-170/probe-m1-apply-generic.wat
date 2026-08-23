;; Can a body APPLY a value whose type is a generic param W (monomorphized to a concrete
;; Fn at the call)? If yes, a generic-W surface work-fn could serve both tiers.

(:wat::core::defn :probe::apply-it :- [W I O] [f <- :W  x <- :I] -> :O (f x))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [d (:probe::apply-it (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* n 2)) 5)]
    (:wat::kernel::println d)))
