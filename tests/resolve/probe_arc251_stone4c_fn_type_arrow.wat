(:wat::core::defn :user::c01-sink [h <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64 0)
(:wat::core::defn :user::c01-pass [g <- [wat.type/i64 :-> wat.type/i64]] -> :wat::core::i64 (:user::c01-sink g))
(:wat::core::defn :user::c02-id [h <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64 0)
