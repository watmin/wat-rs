;; tests/function/probe_arc237_8b_regression_cross_plus_bad.wat — NEGATIVE: cross-type +.
;; Per 8a tightening: (+ i64 f64) rejected at check (or via :NoMatchingClause post-8b).
;; startup MUST fail.

(:wat::core::defn :user::bad [] -> :wat::core::f64 (:wat::core::+ 1 2.0))
