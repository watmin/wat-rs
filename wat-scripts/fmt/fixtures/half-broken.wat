(:wat::core::defn :fix::hb [x <- wat.type/i64] -> wat.type/i64
  (:wat::core::match x
    [n n] [_ 0]))
