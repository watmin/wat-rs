(wat.core/defrecord usr.Shape/Circle
  [r :- wat.type/i64])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (usr.Shape/Circle :r 2)))
