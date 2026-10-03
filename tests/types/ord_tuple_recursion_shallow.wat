;; ord_tuple_recursion_shallow.wat — (1,X) < (2,Y): second element never inspected
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/<
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 1 "anything-here")
    (wat.type/Tuple :- [wat.type/i64 wat.type/String] 2 "anything-there")))
