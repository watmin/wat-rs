;; tests/function/variadic_define_non_vector_rest.wat — NEGATIVE fixture.
;; Rest-binder type MUST be (Vector :- [T]); bare :wat::core::i64 should be rejected.

(:wat::core::defn :my::bogus [& xs <- :wat::core::i64] -> :wat::core::i64 xs)

