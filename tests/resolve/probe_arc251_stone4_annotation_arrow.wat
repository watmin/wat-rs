(:wat::core::defn :user::inc-c01 [x :- wat.type/i64] :- wat.type/i64 (:wat::i64::+ x 1))
(:wat::core::defn :user::inc-c02 [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))
