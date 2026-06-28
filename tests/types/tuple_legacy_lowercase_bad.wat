;; tuple_legacy_lowercase_bad.wat — legacy lowercase tuple hits Pattern 2 poison. Must FAIL.
(:wat::core::defn :my::probe [] -> :(wat::core::i64,wat::core::i64,wat::core::i64) (:wat::core::tuple 1 2 3))
