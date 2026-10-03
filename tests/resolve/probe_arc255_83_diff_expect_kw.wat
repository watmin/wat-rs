(wat.core/defn user/p [] :- wat.type/i64
  (wat.core.Option/expect (wat.core/Option.Some {:value 7}) "missing"))
