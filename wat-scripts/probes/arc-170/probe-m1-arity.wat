;; Does a param typed [I :-> O] accept a 2-arg fn? And can a param be typed generically
;; (a bare type-param W) to accept any-arity fn, then reified via fn-forms?

(:wat::core::defn :probe::takes-1 [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64] -> :wat::core::i64
  (f 3))

(:wat::core::defn :probe::takes-generic :- [W] [f <- :W] -> :wat::core::i64 7)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [two (:wat::core::fn [a <- :wat::core::i64  b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a b))
     ;; pass a 2-arg fn to a generic param (should be fine)
     g   (:probe::takes-generic two)]
    (:wat::kernel::println g)))
