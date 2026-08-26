;; tests/function/variadic_define_type_err.wat — NEGATIVE fixture.
;; Rest declared as (Vector :- [i64]) but caller passes a string in rest position.

(:wat::core::defn :my::sum-of [init <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                (:wat::i64::+ acc x))
              init
              xs))

(:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::sum-of 10 1 "two" 3))

