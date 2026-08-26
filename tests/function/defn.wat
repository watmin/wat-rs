;; tests/function/defn.wat — combined positive fixture for defn tests (arc 166 slice 1).
;; All positive tests (1-7, 10) with uniquely-named compute functions.

;; T1: simple defn — add(2,3)=5
(:wat::core::defn :my::add_t1
  [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x y))

(:wat::core::defn :my::compute_t1 [] -> :wat::core::i64 (:my::add_t1 2 3))

;; T2: recursive defn — fact(5)=120
(:wat::core::defn :my::fact
  [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) 
    1
    (:wat::i64::* n (:my::fact (:wat::i64::- n 1)))))

(:wat::core::defn :my::compute_t2 [] -> :wat::core::i64 (:my::fact 5))

;; T3: defn at top-level position (structural check — just needs to freeze)
(:wat::core::defn :user::double
  [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* x 2))

;; T4: defn inside top-level do
(:wat::core::do
  (:wat::core::defn :user::inc
    [x <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::+ x 1))
  (:wat::core::defn :user::dec
    [x <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::- x 1)))

(:wat::core::defn :my::compute_t4 [] -> :wat::core::i64 (:user::inc (:user::dec 10)))

;; T5: defn inside top-level let body
(:wat::core::let
  [offset 10]
  (:wat::core::defn :user::add-offset
    [x <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::+ x offset)))

(:wat::core::defn :my::compute_t5 [] -> :wat::core::i64 (:user::add-offset 5))

;; T6: defn inside if branch — startup succeeds after Gap I-B
;; (the if-branch defns may or may not register; startup just must not fail)
(:wat::core::if true
  
  (:wat::core::defn :user::f_t6
    [x <- :wat::core::i64] -> :wat::core::i64
    x)
  (:wat::core::defn :user::g_t6
    [x <- :wat::core::i64] -> :wat::core::i64
    x))

;; T7: zero-arg defn
(:wat::core::defn :user::forty-two
  [] -> :wat::core::i64
  42)

(:wat::core::defn :my::compute_t7 [] -> :wat::core::i64 (:user::forty-two))

;; T10: reflection — lookup-define :my::add_t10 returns Some
(:wat::core::defn :my::add_t10
  [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ x y))

(:wat::core::defn :my::compute_t10 [] -> :wat::core::i64
  (:wat::core::match
              (:wat::runtime::lookup-define :my::add_t10)
              
              ((:wat::core::Some _) 1)
              (:wat::core::None    0)))
