;; Co-located fixture for probe_rational_C5c_nan_unordered.rs — arc 300 C5c.
;; docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-C5c-no-warts-NaN-is-unordered.md governs.
;; NaN is produced by division, not a literal: `##NaN` is NOT wat syntax.
;; (:wat::core::f64::/ 0.0 0.0) => nan, (.../ 1.0 0.0) => +inf, (.../ -1.0 0.0) => -inf.

;; row 1/4 — NaN on the right, polymorphic < and >. Correct pre-stone; must stay correct.
(:wat::core::defn :probe::row1-lt [] -> :wat::core::bool (:wat::core::< 1 (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row4-gt [] -> :wat::core::bool (:wat::core::> 1 (:wat::core::f64::/ 0.0 0.0))) ; -> false

;; row 2/3 — the defect: <= and >= with NaN on the right. Were the wart (true); now false.
(:wat::core::defn :probe::row2-le [] -> :wat::core::bool (:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row3-ge [] -> :wat::core::bool (:wat::core::>= 1 (:wat::core::f64::/ 0.0 0.0))) ; -> false

;; row 5 — NaN on the LEFT, all four ops.
(:wat::core::defn :probe::row5-lt [] -> :wat::core::bool (:wat::core::< (:wat::core::f64::/ 0.0 0.0) 1)) ; -> false
(:wat::core::defn :probe::row5-le [] -> :wat::core::bool (:wat::core::<= (:wat::core::f64::/ 0.0 0.0) 1)) ; -> false (was true)
(:wat::core::defn :probe::row5-gt [] -> :wat::core::bool (:wat::core::> (:wat::core::f64::/ 0.0 0.0) 1)) ; -> false
(:wat::core::defn :probe::row5-ge [] -> :wat::core::bool (:wat::core::>= (:wat::core::f64::/ 0.0 0.0) 1)) ; -> false (was true)

;; row 6 — NaN vs NaN.
(:wat::core::defn :probe::row6-lt [] -> :wat::core::bool (:wat::core::< (:wat::core::f64::/ 0.0 0.0) (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row6-le [] -> :wat::core::bool (:wat::core::<= (:wat::core::f64::/ 0.0 0.0) (:wat::core::f64::/ 0.0 0.0))) ; -> false (was true)

;; row 7 — `=`/`not=` untouched: category-aware `values_equal`, never consults an ordering.
;; `(not= 1 NaN)` is `true` in IEEE — the one exception the standard grants.
(:wat::core::defn :probe::row7-noteq [] -> :wat::core::bool (:wat::core::not= 1 (:wat::core::f64::/ 0.0 0.0))) ; -> true
(:wat::core::defn :probe::row7-eq [] -> :wat::core::bool (:wat::core::= 1 (:wat::core::f64::/ 0.0 0.0))) ; -> false

;; row 8 — +/-inf unchanged.
(:wat::core::defn :probe::row8-lt-inf [] -> :wat::core::bool (:wat::core::< 1 (:wat::core::f64::/ 1.0 0.0))) ; -> true
(:wat::core::defn :probe::row8-le-inf [] -> :wat::core::bool (:wat::core::<= 1 (:wat::core::f64::/ 1.0 0.0))) ; -> true

;; row 9 — C5b's exactness intact: must not regress.
(:wat::core::defn :probe::row9-exact [] -> :wat::core::bool (:wat::core::< 9007199254740992.0 9007199254740993)) ; -> true

;; row 10 — non-numeric orderings byte-identical (reach eval_compare through NumOrd::NotNumeric,
;; fall through unchanged to the existing values_compare path).
(:wat::core::defn :probe::row10-string [] -> :wat::core::bool (:wat::core::< "abc" "abd")) ; -> true
(:wat::core::defn :probe::row10-bool [] -> :wat::core::bool (:wat::core::< false true)) ; -> true
(:wat::core::defn :probe::row10-keyword [] -> :wat::core::bool (:wat::core::< :a :b)) ; -> true
(:wat::core::defn :probe::row10-vec [] -> :wat::core::bool (:wat::core::< [1 2] [1 3])) ; -> true
(:wat::core::defn :probe::row10-option [] -> :wat::core::bool (:wat::core::< :wat::core::None (:wat::core::Some 1))) ; -> true

;; row 11 — per-type spellings agree with the polymorphic ones on every NaN row.
;; f64:: routes through the separate `eval_f64_compare` (direct IEEE predicates on raw f64), which
;; was already NaN-correct before this stone; asserted here so it's on record as agreeing, not assumed.
(:wat::core::defn :probe::row11-f64-lt [] -> :wat::core::bool (:wat::core::f64::< 1.0 (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row11-f64-le [] -> :wat::core::bool (:wat::core::f64::<= 1.0 (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row11-f64-gt [] -> :wat::core::bool (:wat::core::f64::> 1.0 (:wat::core::f64::/ 0.0 0.0))) ; -> false
(:wat::core::defn :probe::row11-f64-ge [] -> :wat::core::bool (:wat::core::f64::>= 1.0 (:wat::core::f64::/ 0.0 0.0))) ; -> false

;; row 13 (C5b's own numbering) — ordinary small mixed numerics, unaffected by this stone.
(:wat::core::defn :probe::row13a [] -> :wat::core::bool (:wat::core::< 1 2.0)) ; -> true
(:wat::core::defn :probe::row13b [] -> :wat::core::bool (:wat::core::< 2.0 1)) ; -> false
