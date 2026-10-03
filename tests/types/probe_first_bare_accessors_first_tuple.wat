;; first (bare) on a Tuple — already bare-total (regression guard, green at HEAD).
(wat.core/defn p/f [] :- wat.type/i64 (wat.core/first (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 10 20)))
