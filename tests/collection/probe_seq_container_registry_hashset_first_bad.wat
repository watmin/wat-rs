;; NEGATIVE fixture — must fail to type-check (HashSet is unordered; first is ∅ N/A).
(:wat::core::defn :p::f [] -> :wat::core::i64
  (:wat::core::first (:wat::core::HashSet :wat::core::i64 10 20 30)))

