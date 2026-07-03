;; Co-located fixture for probe_rational_C5_mixed_compare.rs — arc 300 C5.
;; Mixed-numeric comparison/equality must TYPE-CHECK (consistent with eval + clj + C4's
;; mixed arithmetic). At HEAD the checker rejects these (237.8a deleted the cross-numeric
;; path in infer_equality), so this fixture fails to load — the RED.

(:wat::core::defn :probe::lt [] -> :wat::core::bool (:wat::core::< 1 2.0))        ; i64 < f64  => true
(:wat::core::defn :probe::eq [] -> :wat::core::bool (:wat::core::= 1 1.0))        ; = i64 f64  => false (category-aware)
(:wat::core::defn :probe::le-big [] -> :wat::core::bool (:wat::core::<= 1 2N))    ; i64 <= bigint => true
(:wat::core::defn :probe::gt-rat [] -> :wat::core::bool (:wat::core::> 3.0 1/2))  ; f64 > rational => true
