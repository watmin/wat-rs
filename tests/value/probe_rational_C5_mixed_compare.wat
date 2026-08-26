;; Co-located fixture for probe_rational_C5_mixed_compare.rs — arc 300 C5.
;; Mixed-numeric comparison/equality must TYPE-CHECK (consistent with eval + clj + C4's
;; mixed arithmetic). At HEAD the checker rejects these (237.8a deleted the cross-numeric
;; path in infer_equality), so this fixture fails to load — the RED.

(:wat::core::defn :probe::lt [] -> :wat::core::bool (:wat::core::< 1 2.0))        ; i64 < f64  => true
(:wat::core::defn :probe::eq [] -> :wat::core::bool (:wat::core::= 1 1.0))        ; = i64 f64  => false (category-aware)
(:wat::core::defn :probe::le-big [] -> :wat::core::bool (:wat::core::<= 1 2N))    ; i64 <= bigint => true
(:wat::core::defn :probe::gt-rat [] -> :wat::core::bool (:wat::core::> 3.0 1/2))  ; f64 > rational => true

;; ── Stone C5b gate — docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-C5b-exact-mixed-numeric-order.md
;; Mixed-numeric ORDERING through the one exact door. 2^53 = 9007199254740992 is the last integer f64
;; represents exactly; 2^53+1 = 9007199254740993 rounds to 2^53.0 under the old coerce-to-f64 arms, so the
;; two operands compared EQUAL and every RED row below returned the wrong answer.

;; row 2/3/4/6 — the i64<->f64 boundary case itself, all four ordering ops, both directions.
(:wat::core::defn :probe::c5b-row2 [] -> :wat::core::bool (:wat::core::< 9007199254740992.0 9007199254740993))  ; RED at HEAD -> true
(:wat::core::defn :probe::c5b-row3 [] -> :wat::core::bool (:wat::core::< 9007199254740993 9007199254740992.0)) ; green by accident -> false
(:wat::core::defn :probe::c5b-row4 [] -> :wat::core::bool (:wat::core::> 9007199254740993 9007199254740992.0)) ; RED at HEAD -> true
(:wat::core::defn :probe::c5b-row5 [] -> :wat::core::bool (:wat::core::<= 9007199254740992.0 9007199254740993)) ; green by accident -> true
(:wat::core::defn :probe::c5b-row6 [] -> :wat::core::bool (:wat::core::>= 9007199254740992.0 9007199254740993)) ; RED at HEAD -> false

;; row 7 — `=` is category-aware (C4) and structurally immune; unchanged by this stone.
(:wat::core::defn :probe::c5b-row7 [] -> :wat::core::bool (:wat::core::= 9007199254740992.0 9007199254740993)) ; green -> false

;; row 8 — the other two lossy pairs (bigint<->f64, rational<->f64) below 2^53; already right, must stay right.
(:wat::core::defn :probe::c5b-row8a [] -> :wat::core::bool (:wat::core::< 1N 2.0)) ; green -> true
(:wat::core::defn :probe::c5b-row8b [] -> :wat::core::bool (:wat::core::> 3.0 1/2)) ; green -> true

;; row 9 — the BigInt<->f64 mirror of row 2/3/4, using an explicit BigInt (N suffix) above 2^53.
(:wat::core::defn :probe::c5b-row9a [] -> :wat::core::bool (:wat::core::< 9007199254740992.0 9007199254740993N)) ; RED at HEAD -> true
(:wat::core::defn :probe::c5b-row9b [] -> :wat::core::bool (:wat::core::< 9007199254740993N 9007199254740992.0)) ; green by accident -> false
(:wat::core::defn :probe::c5b-row9c [] -> :wat::core::bool (:wat::core::> 9007199254740993N 9007199254740992.0)) ; RED at HEAD -> true

;; row 10 — the Rational<->f64 mirror. 18014398509481985/2 = 9007199254740992.5, a genuinely fractional
;; value (denominator 2, not reducible to an integer) above 2^53 that f64 cannot represent exactly (the
;; nearest doubles at this magnitude are 2 apart) — proves the rational side is compared EXACTLY, not
;; coerced down and rounded onto the f64 operand.
(:wat::core::defn :probe::c5b-row10a [] -> :wat::core::bool (:wat::core::< 9007199254740992.0 18014398509481985/2)) ; RED at HEAD -> true
(:wat::core::defn :probe::c5b-row10b [] -> :wat::core::bool (:wat::core::< 18014398509481985/2 9007199254740992.0)) ; green by accident -> false
(:wat::core::defn :probe::c5b-row10c [] -> :wat::core::bool (:wat::core::> 18014398509481985/2 9007199254740992.0)) ; RED at HEAD -> true

;; row 11 — +/-inf must survive the exact path (produced by division; `##Inf`/`##NaN` are NOT wat literals).
(:wat::core::defn :probe::c5b-row11-inf [] -> :wat::core::bool (:wat::core::< 1 (:wat::f64::/ 1.0 0.0)))     ; green, must stay -> true
(:wat::core::defn :probe::c5b-row11-neg-inf [] -> :wat::core::bool (:wat::core::> 1 (:wat::f64::/ -1.0 0.0))) ; green, must stay -> true

;; row 12 — was NaN policy PRESERVED exactly, wart and all, under C5b (`values_compare` maps NaN -> Equal,
;; so `<=` read Equal as true). SUPERSEDED by DESIGN-STONE-C5c-no-warts-NaN-is-unordered.md: `eval_compare`
;; now consults `numeric_order` first and returns false for all four ops on `Incomparable`. `values_compare`
;; itself is still unchanged (that collection-totality seam stays); this row exercises `eval_compare`'s
;; corrected policy.
(:wat::core::defn :probe::c5b-row12-nan-lt [] -> :wat::core::bool (:wat::core::< 1 (:wat::f64::/ 0.0 0.0)))  ; green, must stay -> false
(:wat::core::defn :probe::c5b-row12-nan-le [] -> :wat::core::bool (:wat::core::<= 1 (:wat::f64::/ 0.0 0.0))) ; C5c: -> false (was the wart, true; superseded)

;; row 13 — ordinary small mixed numerics, unaffected by the fix.
(:wat::core::defn :probe::c5b-row13a [] -> :wat::core::bool (:wat::core::< 1 2.0)) ; green, must stay -> true
(:wat::core::defn :probe::c5b-row13b [] -> :wat::core::bool (:wat::core::< 2.0 1)) ; green, must stay -> false
