;; tests/types/probe_arc237_8d_equality_intrinsic.wat
;; Co-located fixture for probe_arc237_8d_equality_intrinsic.rs
;; Loaded via startup_beside(file!()). Positive regression cases only.
;; Negative check-error cases use separate .wat.bad fixtures.

;; Shared type for regression_eq_records_is_the_relational_case
(:wat::core::defrecord :my::Pt [x <- wat.type/i64  y <- wat.type/i64])

;; regression_eq_scalars (5 assertions)
(:wat::core::defn :user::eq-scalars-i64-eq [] -> wat.type/bool (:wat::core::= 1 1))
(:wat::core::defn :user::eq-scalars-i64-neq [] -> wat.type/bool (:wat::core::= 1 2))
(:wat::core::defn :user::eq-scalars-f64-eq [] -> wat.type/bool (:wat::core::= 1.0 1.0))
(:wat::core::defn :user::eq-scalars-str-eq [] -> wat.type/bool (:wat::core::= "a" "a"))
(:wat::core::defn :user::eq-scalars-bool-neq [] -> wat.type/bool (:wat::core::= true false))

;; regression_eq_composites_recursive (2 assertions)
(:wat::core::defn :user::eq-composites-equal [] -> wat.type/bool (:wat::core::= [1 2 3] [1 2 3]))
(:wat::core::defn :user::eq-composites-diff-len [] -> wat.type/bool (:wat::core::= [1 2] [1 2 3]))

;; regression_not_eq (2 assertions)
(:wat::core::defn :user::not-eq-neq [] -> wat.type/bool (:wat::core::not= 1 2))
(:wat::core::defn :user::not-eq-eq [] -> wat.type/bool (:wat::core::not= 1 1))

;; regression_eq_records_is_the_relational_case (2 assertions)
(:wat::core::defn :user::eq-records-equal [] -> wat.type/bool (:wat::core::= (:my::Pt :x 0 :y 0) (:my::Pt :x 0 :y 0)))
(:wat::core::defn :user::eq-records-diff [] -> wat.type/bool (:wat::core::= (:my::Pt :x 0 :y 0) (:my::Pt :x 0 :y 9)))

;; per-type-equality-restored.md (2026-08-05) — 237.8d's per-Type equality cut
;; REVERSED. `i64::=`/`i64::not=`/`f64::=`/`f64::not=` restored beside their ordering
;; twins (`i64::>` etc), which the cut never touched. restored_i64_eq /
;; restored_i64_not_eq / restored_f64_eq / restored_f64_not_eq (7 assertions,
;; including the f64 NaN case — NaN != NaN falls out of eval_f64_compare for free,
;; not special-cased).
(:wat::core::defn :user::restored-i64-eq-true [] -> wat.type/bool (:wat::i64::= 1 1))
(:wat::core::defn :user::restored-i64-eq-false [] -> wat.type/bool (:wat::i64::= 1 2))
(:wat::core::defn :user::restored-i64-not-eq-true [] -> wat.type/bool (:wat::i64::not= 1 2))
(:wat::core::defn :user::restored-i64-not-eq-false [] -> wat.type/bool (:wat::i64::not= 1 1))
(:wat::core::defn :user::restored-f64-eq-true [] -> wat.type/bool (:wat::f64::= 1.5 1.5))
(:wat::core::defn :user::restored-f64-eq-false [] -> wat.type/bool (:wat::f64::= 0.0 1.0))
(:wat::core::defn :user::restored-f64-not-eq-true [] -> wat.type/bool (:wat::f64::not= 1.5 2.5))
(:wat::core::defn :user::restored-f64-nan-not-eq-itself [] -> wat.type/bool (:wat::f64::not= (:wat::f64::/ 0.0 0.0) (:wat::f64::/ 0.0 0.0)))
