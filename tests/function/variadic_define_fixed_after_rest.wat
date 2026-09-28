;; tests/function/variadic_define_fixed_after_rest.wat — NEGATIVE fixture.
;; Fixed param after rest binder — Runtime MalformedForm expected.

(:wat::core::defn :my::bogus [init <- wat.type/i64 & xs <- (wat.type/Vector :- [wat.type/i64]) extra <- wat.type/i64] -> wat.type/i64 init)

