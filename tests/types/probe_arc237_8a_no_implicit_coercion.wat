;; tests/types/probe_arc237_8a_no_implicit_coercion.wat
;; Co-located fixture for probe_arc237_8a_no_implicit_coercion.rs
;; Loaded via startup_beside(file!()). Positive (same-type) cases only.
;; Negative (cross-type) cases use separate _bad.wat fixtures.

(:wat::core::defn :user::arith-i64-same [] -> :wat::core::i64 (:wat::core::+ 1 2))
(:wat::core::defn :user::arith-f64-same [] -> :wat::core::f64 (:wat::core::+ 1.0 2.0))
(:wat::core::defn :user::arith-variadic-same [] -> :wat::core::i64 (:wat::core::+ 1 2 3))
(:wat::core::defn :user::cmp-i64-same [] -> :wat::core::bool (:wat::core::< 1 2))
(:wat::core::defn :user::cmp-f64-same [] -> :wat::core::bool (:wat::core::< 1.0 2.0))
(:wat::core::defn :user::cmp-str-same [] -> :wat::core::bool (:wat::core::= "a" "a"))
