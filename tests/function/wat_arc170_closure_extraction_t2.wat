;; T2: top-level defn calls other top-level defns.
(:wat::core::defn :my::times-two [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :my::times-four [n <- wat.type/i64] -> wat.type/i64 (:my::times-two (:my::times-two n)))
