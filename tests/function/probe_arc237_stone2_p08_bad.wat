;; tests/function/probe_arc237_stone2_p08_bad.wat — NEGATIVE probe 8: no matching clause at call.
;; Calling :i64-typed clause with :String arg must reject at check time. startup MUST fail.

(:wat::core::defclause :my::only-i64
  ([x <- :wat::core::i64] -> :wat::core::i64 x))
(:wat::core::defn :user::bad [] -> :wat::core::nil
  (:wat::core::do (:my::only-i64 "string-arg") nil))
