;; Vector of Op.Mark, filtered by a pred over Op. REFUSED at step 1, ACCEPTED at step 2.
(wat.core/defenum u/Op wat.enum/Pure
  :Mark [n :- wat.type/i64]
  :Other [])
(wat.core/defn u/is-op [o :- u/Op] :- wat.type/bool true)
(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let [_ (wat.core/filter u/is-op [(u/Op.Mark {:n 1})])]
    (wat.kernel/println "ok")))
