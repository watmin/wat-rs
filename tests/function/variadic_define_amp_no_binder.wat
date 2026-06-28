;; tests/function/variadic_define_amp_no_binder.wat — NEGATIVE fixture.
;; `&` without a binder — Runtime MalformedForm expected.

(:wat::core::defn :my::bogus [init <- :wat::core::i64 &] -> :wat::core::i64 init)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
