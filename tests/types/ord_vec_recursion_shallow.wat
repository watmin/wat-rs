;; ord_vec_recursion_shallow.wat — [9,1,1] > [1,99,99]: first element wins
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/>
    (wat.type/Vector :- [wat.type/i64] 9 1 1)
    (wat.type/Vector :- [wat.type/i64] 1 99 99)))
