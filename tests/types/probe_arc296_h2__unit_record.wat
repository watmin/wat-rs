(wat.core/defrecord usr.Shape/Nothing [])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (usr.Shape/Nothing)))
