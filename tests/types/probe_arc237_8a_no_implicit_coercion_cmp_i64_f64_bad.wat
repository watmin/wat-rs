;; Formerly a negative fixture (i64 < f64 rejected at check, no implicit coercion in
;; comparison). Arc 300 C5 retired 237.8a's comparison-side reject — mixed-numeric
;; comparison now type-checks (consistency with C4's arithmetic + eval + clj). This
;; fixture now type-checks; kept at its original path (name unchanged) since the test
;; (`comparison_i64_f64_mixed_coerces`) now asserts Ok.
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::< 1 2.0))
