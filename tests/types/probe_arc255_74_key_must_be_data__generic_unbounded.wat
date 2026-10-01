;; AMEND-255.74 D3 — the same function as generic_bounded.wat, with the `[T :< Equatable]`
;; bound dropped: must be refused, naming T, not some unrelated error.
(:wat::core::defn :user::singleton :- [T]
  [x <- :T] -> (wat.type/HashSet :- [:T])
  (wat.type/HashSet :- [:T] x))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::core::count (:user::singleton 1))))
