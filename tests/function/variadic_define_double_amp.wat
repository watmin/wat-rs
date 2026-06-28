;; tests/function/variadic_define_double_amp.wat — NEGATIVE fixture.
;; Double `&` in define signature — Runtime MalformedForm expected.

(:wat::core::defn :my::bogus [& _a <- :wat::core::Vector<wat::core::i64> & xs <- :wat::core::Vector<wat::core::i64>] -> :wat::core::i64 0)

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
