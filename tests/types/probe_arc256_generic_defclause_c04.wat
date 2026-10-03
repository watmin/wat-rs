;; probe_arc256_generic_defclause_c04.wat — two distinct instantiations (i64 + bool). RED at HEAD.

(wat.core/defclause user/firstof ([a :- T b :- T] :- T a))
(wat.core/defn user/p-i64  [] :- wat.type/i64  (user/firstof 1 2))
(wat.core/defn user/p-bool [] :- wat.type/bool (user/firstof true false))
