;; tests/function/probe_arc237_8b_regression_cross_lt.wat — formerly NEGATIVE
;; (cross-type <, i64<f64, rejected at check). Arc 300 C5 retired 237.8a's
;; comparison-side reject: mixed-numeric ordering now type-checks (consistency
;; with C4's arithmetic + eval + clj). startup now succeeds; the test
;; (`regression_cross_type_lt_coerces`) asserts Ok. Name/path kept unchanged.

(:wat::core::defn :user::bad [] -> :wat::core::bool (:wat::core::< 1 2.0))
