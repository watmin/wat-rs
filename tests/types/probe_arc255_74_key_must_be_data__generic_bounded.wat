;; AMEND-255.74 D3 — a generic function building a HashSet keyed by its OWN type parameter
;; must declare that parameter `[T :< :wat::core::Equatable]` to check. Positive sibling of
;; generic_unbounded.wat.
(:wat::core::defn :user::singleton :- [[T :< :wat::core::Equatable]]
  [x <- :T] -> (wat.type/HashSet :- [:T])
  (wat.type/HashSet :- [:T] x))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::core::count (:user::singleton 1))))
