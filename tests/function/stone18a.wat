;; tests/function/stone18a.wat — positive fixture for stone18a.rs behavioral-preservation contracts.
;; C01: fn-form with single typed param (double)
;; C02: fn-form with multi-param triple-arrow (compute)

(:wat::core::defn :test::double [n <- :wat::core::i64] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
     (:wat::i64::+ x x))
   n))

(:wat::core::defn :test::compute [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
      (:wat::i64::+ x y))
   a b))

