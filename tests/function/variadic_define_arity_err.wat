;; tests/function/variadic_define_arity_err.wat — NEGATIVE fixture.
;; Caller omits the required fixed param `init`; type checker surfaces ArityMismatch.

(:wat::core::defn :my::sum-of [init <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ acc x))
              init
              xs))

(:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::sum-of))

