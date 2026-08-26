;; T14: three-level transitive dep chain a -> b -> c.
;; Prologue must list a, b, c in topological order.
(:wat::core::defn :my::a [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ n 1))
(:wat::core::defn :my::b [n <- :wat::core::i64] -> :wat::core::i64 (:my::a (:my::a n)))
(:wat::core::defn :my::c [n <- :wat::core::i64] -> :wat::core::i64 (:my::b (:my::b n)))
