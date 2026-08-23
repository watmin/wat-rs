;; tests/function/tco.wat — combined fixture for TCO tests.
;; Each test defines a uniquely-named :user::compute_tN function.

;; T1: self-recursion via if at million depth
(:wat::core::defn :app::countdown [n <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) 
              acc
              (:app::countdown (:wat::core::i64::- n 1) (:wat::core::i64::+ acc 1))))

(:wat::core::defn :user::compute_t1 [] -> :wat::core::i64 (:app::countdown 1000000 0))

;; T2: self-recursion via match at high depth
(:wat::core::defn :app::drain [remaining <- :wat::core::i64 acc <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match
              (:wat::core::if (:wat::core::> remaining 0) 
                (:wat::core::Some remaining)
                :wat::core::None)
              
              ((:wat::core::Some v)
                (:app::drain (:wat::core::i64::- v 1) (:wat::core::i64::+ acc 1)))
              (:wat::core::None acc)))

(:wat::core::defn :user::compute_t2 [] -> :wat::core::i64 (:app::drain 100000 0))

;; T3: mutual recursion
(:wat::core::defn :app::is-even [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::= n 0) 
              true
              (:app::is-odd (:wat::core::i64::- n 1))))

(:wat::core::defn :app::is-odd [n <- :wat::core::i64] -> :wat::core::bool
  (:wat::core::if (:wat::core::= n 0) 
              false
              (:app::is-even (:wat::core::i64::- n 1))))

(:wat::core::defn :user::compute_t3 [] -> :wat::core::bool (:app::is-even 100000))

;; T4: tail call through let body
(:wat::core::defn :app::loop_t4 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
              [next (:wat::core::i64::- n 1)]
              (:wat::core::if (:wat::core::<= n 0) 
                0
                (:app::loop_t4 next))))

(:wat::core::defn :user::compute_t4 [] -> :wat::core::i64 (:app::loop_t4 100000))

;; T5: non-tail recursion at modest depth
(:wat::core::defn :app::pow2 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) 
              1
              (:wat::core::i64::* 2 (:app::pow2 (:wat::core::i64::- n 1)))))

(:wat::core::defn :user::compute_t5 [] -> :wat::core::i64 (:app::pow2 20))

;; T6: try + TailCall coexistence (short-circuits with Ok 0)
(:wat::core::defn :app::check [n <- :wat::core::i64] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::if (:wat::core::< n 0) 
              (:wat::core::Err "negative")
              (:wat::core::Ok n)))

(:wat::core::defn :app::loop_t6 [n <- :wat::core::i64] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
              [valid (:wat::core::Result/try (:app::check n))]
              (:wat::core::if (:wat::core::= valid 0) 
                (:wat::core::Ok 0)
                (:app::loop_t6 (:wat::core::i64::- valid 1)))))

(:wat::core::defn :user::compute_t6 [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String]) (:app::loop_t6 50000))

;; T7: try propagates Err (starts at -1)
(:wat::core::defn :app::loop_t7 [n <- :wat::core::i64] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::let
              [valid (:wat::core::Result/try (:app::check n))]
              (:wat::core::if (:wat::core::<= valid (:wat::core::i64::- 0 1)) 
                (:wat::core::Ok 0)
                (:app::loop_t7 (:wat::core::i64::- valid 1)))))

;; Start at -1 so `check` immediately returns Err and `try` propagates.
(:wat::core::defn :user::compute_t7 [] -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String]) (:app::loop_t7 -1))

;; T8: fn-valued tail call via let-bound symbol
(:wat::core::defn :user::compute_t8 [] -> :wat::core::i64
  (:wat::core::let
              [f
                (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
                  (:wat::core::if (:wat::core::= n 0)  0 n))]
              (f 42)))

;; T9: inline fn literal tail call
(:wat::core::defn :user::compute_t9 [] -> :wat::core::i64
  ((:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
               (:wat::core::i64::* n 2))
             21))

;; T10: named define tail-calls fn param
(:wat::core::defn :app::invoke [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64 n <- :wat::core::i64] -> :wat::core::i64 (f n))

(:wat::core::defn :user::compute_t10 [] -> :wat::core::i64
  (:wat::core::let
              [double
                (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                  (:wat::core::i64::* x 2))]
              (:app::invoke double 21)))

;; T11: inline fn + named alternation at high depth
(:wat::core::defn :app::go [state <- :wat::core::i64 n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) 
              state
              ((:wat::core::fn [s <- :wat::core::i64 k <- :wat::core::i64] -> :wat::core::i64
                 (:app::go (:wat::core::i64::+ s 1) (:wat::core::i64::- k 1)))
               state n)))

(:wat::core::defn :user::compute_t11 [] -> :wat::core::i64 (:app::go 0 100000))
