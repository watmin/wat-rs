;; tests/function/probe_arc237_8b_defclause_arithmetic.wat
;; Arc 237 Stone 237.8b — recipe-lock + numeric grid via wat-defclause.
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *.wat.bad files.

;; GATE 1 — defclause supports & rest-binders in args-vec
(:wat::core::defclause :my::g1-sum-all
  ([first <- :wat::core::i64
    & rest <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::i64
    (:wat::core::foldl
      (:wat::core::fn [acc <- :wat::core::i64
                       n <- :wat::core::i64] -> :wat::core::i64
        (:wat::i64::+ acc n))
      first
      rest)))
(:wat::core::defn :user::gate-1-sum-all [] -> :wat::core::i64 (:my::g1-sum-all 1 2 3 4))

;; GATE 2 — defclause first-match dispatches by arg-<--Type (no :guard)
(:wat::core::defclause :my::g2-label
  ([x <- :wat::core::i64] -> :wat::core::String "i64")
  ([x <- :wat::core::f64] -> :wat::core::String "f64"))
(:wat::core::defn :user::gate-2-dispatch [] -> :wat::core::String (:my::g2-label 42))

;; GATE 3 — 0-ary clause body literal 0 infers as :i64
(:wat::core::defclause :my::g3-default
  ([] -> :wat::core::i64 0))
(:wat::core::defn :user::gate-3-zero-ary [] -> :wat::core::i64 (:my::g3-default))

;; GATE 4a — i64 ordering primitives
(:wat::core::defn :user::gate-4a-lt [] -> :wat::core::bool (:wat::i64::< 1 2))
(:wat::core::defn :user::gate-4a-gt [] -> :wat::core::bool (:wat::i64::> 5 3))

;; GATE 4b — f64 NaN ordering (0.0 / 0.0 produces NaN; 1.0 < NaN is false per IEEE 754)
(:wat::core::defn :user::gate-4b-nan [] -> :wat::core::bool
  (:wat::f64::< 1.0 (:wat::f64::/ 0.0 0.0)))

;; REGRESSION — existing arithmetic + ordering behavior
(:wat::core::defn :user::regression-i64-2ary [] -> :wat::core::i64 (:wat::core::+ 1 2))
(:wat::core::defn :user::regression-f64-2ary [] -> :wat::core::f64 (:wat::core::+ 1.0 2.0))
(:wat::core::defn :user::regression-variadic-3 [] -> :wat::core::i64 (:wat::core::+ 1 2 3))
(:wat::core::defn :user::regression-minus-negate [] -> :wat::core::i64 (:wat::core::- 5))
(:wat::core::defn :user::regression-lt [] -> :wat::core::bool (:wat::core::< 1 2))

;; MINT-CONFIRMERS — new primitives minted as part of Stone 237.8b
(:wat::core::defn :user::mint-i64-lte-boundary [] -> :wat::core::bool (:wat::i64::<= 5 5))
(:wat::core::defn :user::mint-i64-lte-false [] -> :wat::core::bool (:wat::i64::<= 5 3))
(:wat::core::defn :user::mint-f64-lt [] -> :wat::core::bool (:wat::f64::< 1.0 2.0))
(:wat::core::defn :user::mint-f64-gte [] -> :wat::core::bool (:wat::f64::>= 5.0 5.0))
(:wat::core::defn :user::mint-not-eq [] -> :wat::core::bool (:wat::core::not= 1 2))
(:wat::core::defn :user::mint-plus-zero-ary [] -> :wat::core::i64 (:wat::core::+))
(:wat::core::defn :user::mint-star-zero-ary [] -> :wat::core::i64 (:wat::core::*))
