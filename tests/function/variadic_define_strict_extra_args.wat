;; tests/function/variadic_define_strict_extra_args.wat — NEGATIVE fixture.
;; A strict-arity define rejects extra args; variadic branch must NOT fire.

(:wat::core::defn :my::add [a <- wat.type/i64 b <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ a b))

(:wat::core::defn :user::compute [] -> wat.type/i64 (:my::add 40 2 99))

