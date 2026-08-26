;; tests/function/variadic_define_strict_extra_args.wat — NEGATIVE fixture.
;; A strict-arity define rejects extra args; variadic branch must NOT fire.

(:wat::core::defn :my::add [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ a b))

(:wat::core::defn :user::compute [] -> :wat::core::i64 (:my::add 40 2 99))

