(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println (wat.deporder/verify-stdlib)))
