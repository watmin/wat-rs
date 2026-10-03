;; RELAND 8 step 3: after Option :None [] repair, does sort-by of the generated
;; Row/sk accessor classify Pure? Empty vector — if the comparator is refused,
;; this raises (store-kill class). If admitted, it prints [].
(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println
    (wat.core/show
      (wat.core/sort-by wat.query.Row/sk
        (wat.type/Vector :- [wat.query/Row]))))
  nil)
