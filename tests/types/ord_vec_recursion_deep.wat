;; ord_vec_recursion_deep.wat — (Vec :- [(Vec :- [i64])]): [[1,2],[3,4]] < [[1,2],[3,5]]
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/<
    (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
      (wat.type/Vector :- [wat.type/i64] 1 2)
      (wat.type/Vector :- [wat.type/i64] 3 4))
    (wat.type/Vector :- [(wat.type/Vector :- [wat.type/i64])]
      (wat.type/Vector :- [wat.type/i64] 1 2)
      (wat.type/Vector :- [wat.type/i64] 3 5))))
