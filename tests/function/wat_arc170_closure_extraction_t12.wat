;; T12: classify using :wat::core::cond substrate macro — body expands to primitives.
(:wat::core::defn :my::classify [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::cond
              ((:wat::core::< n 0) "negative")
              ((:wat::core::= n 0) "zero")
              (:else "positive")))
