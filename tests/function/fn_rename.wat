;; tests/function/fn_rename.wat — combined positive fixture for fn_rename tests.
;; Arc 155 — :wat::core::lambda → :wat::core::fn + bare :fn(...) → :wat::core::Fn(...).
;; All positive tests (2, 4, 5, 6, 8, 9, 10, 11) combined with deconflicted names.
;; Startup MUST succeed for the positive contract.

;; T2: operator-position canonical — :wat::core::fn works
(:wat::core::defn :my::apply_t2 [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
               x)
             5))

;; T4: type-position canonical — :wat::core::Fn(...) works
(:wat::core::defn :user::apply_t4 [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (f x))

(:wat::core::defn :my::invoke_t4 [] -> :wat::core::i64
  (:user::apply_t4
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                x)
              42))

;; T5: :wat::core::fn operator does NOT fire BareLegacyLowercaseFn
(:wat::core::defn :my::apply_t5 [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
               x)
             7))

;; T6: :wat::core::Fn(...) type does NOT fire BareLegacyLowercaseFn
(:wat::core::defn :user::apply_t6 [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (f x))

(:wat::core::defn :my::invoke_t6 [] -> :wat::core::i64
  (:user::apply_t6
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                (:wat::i64::+ x 1))
              10))

;; T8: fn in tail position type-checks
(:wat::core::defn :user::double_t8 [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* n 2))

(:wat::core::defn :my::apply_t8 [] -> :wat::core::i64
  ((:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
               (:user::double_t8 n))
             5))

;; T9: mixed canonical — fn operator + Fn type together
(:wat::core::defn :user::apply_t9 [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (f x))

(:wat::core::defn :my::invoke_t9 [] -> :wat::core::i64
  (:user::apply_t9
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                (:wat::i64::+ x 1))
              5))

;; T10: fn body with let
(:wat::core::defn :my::apply_t10 [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
               (:wat::core::let
                 [a (:wat::i64::+ x 5)
                  b (:wat::i64::* a 2)]
                 b))
             3))

;; T11: reflection — :wat::core::fn registry entry exists
(:wat::core::defn :my::apply_t11 [] -> :wat::core::i64
  ((:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
               (:wat::i64::+ a b))
             10 20))

