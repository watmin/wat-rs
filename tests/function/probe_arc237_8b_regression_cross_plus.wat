;; tests/function/probe_arc237_8b_regression_cross_plus.wat — cross-type +.
;; arc 300 C4: mixed contagion adopted — (+ 1 2.0) => f64; this now type-checks.
;; (237.8a's blanket arithmetic reject was retired once N-ary became an honest gap.)

(:wat::core::defn :user::bad [] -> :wat::core::f64 (:wat::core::+ 1 2.0))
