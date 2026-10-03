;; tests/function/probe_arc241_stone5_c05.wat — NEGATIVE contract 5: rest element type mismatch.
;; Passing "three" (String) where (Vector :- [i64]) element is expected. Startup SUCCEEDS —
;; rest element types are checked at dispatch, so the error arrives at EVAL (contract_05 invokes).

(wat.core/defclause my/sum-all
  ([first :- wat.type/i64
    & rest :- (wat.type/Vector :- [wat.type/i64])] :- wat.type/i64
    (wat.core/foldl
      (wat.core/fn [acc :- wat.type/i64
                       n :- wat.type/i64] :- wat.type/i64
        (wat.i64/+ acc n))
      first
      rest)))
(wat.core/defn user/bad [] :- wat.type/i64 (my/sum-all 1 2 "three"))
