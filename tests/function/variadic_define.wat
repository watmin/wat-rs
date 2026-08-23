;; tests/function/variadic_define.wat — combined positive fixture for variadic define tests.
;; Each test's compute function is named :user::compute_tN to avoid conflicts.

;; T1-T4: :my::sum-of (fixed init + variadic xs)
(:wat::core::defn :my::sum-of [init <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ acc x))
              init
              xs))

;; T1: zero rest-args → 100
(:wat::core::defn :user::compute_t1 [] -> :wat::core::i64 (:my::sum-of 100))

;; T2: one rest-arg → 15
(:wat::core::defn :user::compute_t2 [] -> :wat::core::i64 (:my::sum-of 10 5))

;; T3: many rest-args → 115
(:wat::core::defn :user::compute_t3 [] -> :wat::core::i64 (:my::sum-of 100 1 2 3 4 5))

;; T4, T5: :my::sum (no fixed params, only rest)
(:wat::core::defn :my::sum [& xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ acc x))
              0
              xs))

;; T4: 7+8+9+10 = 34
(:wat::core::defn :user::compute_t4 [] -> :wat::core::i64 (:my::sum 7 8 9 10))

;; T5: zero args → 0
(:wat::core::defn :user::compute_t5 [] -> :wat::core::i64 (:my::sum))

;; T6: rest binding is a real Vec — count-rest
(:wat::core::defn :my::count-rest [init <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64 (:wat::core::length xs))

(:wat::core::defn :user::compute_t6 [] -> :wat::core::i64 (:my::count-rest 999 10 20 30))

;; T9 (reflection): :my::sum-of is already defined above; compute_t9 renders signature.
(:wat::core::defn :user::compute_t9 [] -> :wat::core::String
  (:wat::core::let
              [sig-opt
                (:wat::runtime::signature-of-defn :my::sum-of)
               rendered
                (:wat::edn::write sig-opt)]
              rendered))

;; T10 (canonical foldl pattern): :my::add-all
(:wat::core::defn :my::add-all [seed <- :wat::core::i64 & xs <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::foldl
              (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                (:wat::core::i64::+ acc x))
              seed
              xs))

;; T10: 0+1+2+...+10 = 55
(:wat::core::defn :user::compute_t10 [] -> :wat::core::i64 (:my::add-all 0 1 2 3 4 5 6 7 8 9 10))

;; T13 (regression: strict-arity define unchanged)
(:wat::core::defn :my::add_strict [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ a b))

(:wat::core::defn :user::compute_t13 [] -> :wat::core::i64 (:my::add_strict 40 2))
