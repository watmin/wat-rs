(:wat::core::defn :fix::hb [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match x
    (n n) (_ 0)))
