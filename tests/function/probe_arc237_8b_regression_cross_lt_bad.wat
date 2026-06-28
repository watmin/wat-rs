;; tests/function/probe_arc237_8b_regression_cross_lt_bad.wat — NEGATIVE: cross-type <.
;; (< i64 f64) must reject at check.
;; startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::bool (:wat::core::< 1 2.0))
