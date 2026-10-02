;; T4: inline lambda, no captures (factory pattern returning fn).
(:wat::core::defn :my::factory [] -> [wat.type/i64 :-> wat.type/i64]
  (:wat::core::fn [n <- wat.type/i64] -> wat.type/i64
              (:wat::i64::+ n 7)))
