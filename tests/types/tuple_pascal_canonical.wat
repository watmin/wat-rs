;; tuple_pascal_canonical.wat — canonical PascalCase Tuple constructor.
(:wat::core::defn :my::compute [] -> (wat.type/Tuple :- [wat.type/i64 wat.type/i64 wat.type/i64]) (:wat::core::Tuple 1 2 3))
