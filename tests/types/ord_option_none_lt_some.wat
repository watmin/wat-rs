;; ord_option_none_lt_some.wat — None < Some(_)
(wat.core/defn user/compute [] :- wat.type/bool
  (wat.core/let
    [a wat.core/Option.None
     b (wat.core/Option.Some {:value 0})]
    (wat.core/< a b)))
