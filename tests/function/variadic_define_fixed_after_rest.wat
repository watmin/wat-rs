;; tests/function/variadic_define_fixed_after_rest.wat — NEGATIVE fixture.
;; Fixed param after rest binder — Runtime MalformedForm expected.

(:wat::core::defn :my::bogus [init <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64]) extra <- :wat::core::i64] -> :wat::core::i64 init)

