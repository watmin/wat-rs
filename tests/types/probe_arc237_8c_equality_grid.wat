;; tests/types/probe_arc237_8c_equality_grid.wat
;; Co-located fixture for probe_arc237_8c_equality_grid.rs
;; Loaded via startup_beside(file!()). Positive regression cases only.
;; Negative check-error cases use separate .wat.bad fixtures.

;; regression_eq_scalars (5 assertions)
(:wat::core::defn :user::eq-scalars-i64-eq [] -> :wat::core::bool (:wat::core::= 1 1))
(:wat::core::defn :user::eq-scalars-i64-neq [] -> :wat::core::bool (:wat::core::= 1 2))
(:wat::core::defn :user::eq-scalars-f64-eq [] -> :wat::core::bool (:wat::core::= 1.0 1.0))
(:wat::core::defn :user::eq-scalars-str-eq [] -> :wat::core::bool (:wat::core::= "a" "a"))
(:wat::core::defn :user::eq-scalars-bool-neq [] -> :wat::core::bool (:wat::core::= true false))

;; regression_eq_composites_recursive (2 assertions)
(:wat::core::defn :user::eq-composites-equal [] -> :wat::core::bool (:wat::core::= [1 2 3] [1 2 3]))
(:wat::core::defn :user::eq-composites-diff-len [] -> :wat::core::bool (:wat::core::= [1 2] [1 2 3]))

;; regression_not_eq (2 assertions)
(:wat::core::defn :user::not-eq-neq [] -> :wat::core::bool (:wat::core::not= 1 2))
(:wat::core::defn :user::not-eq-eq [] -> :wat::core::bool (:wat::core::not= 1 1))
