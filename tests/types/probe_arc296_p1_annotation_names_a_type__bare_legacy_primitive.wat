;; CONTROL — the third lexical class: a bare LOWERCASE legacy primitive.
;; Not a type var (lowercase), not FQDN. It lives only in is_builtin_primitive.
(:wat::core::defn :user::f [x <- :i64] -> :i64 x)
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:user::f 7)))
