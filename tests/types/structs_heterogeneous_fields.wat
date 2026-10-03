;; structs_heterogeneous_fields.wat — struct with heterogeneous field types.
(wat.core/defstruct my.market/Tick
  [symbol :- wat.type/String
   price  :- wat.type/f64
   volume :- wat.type/i64])
(wat.core/defn my/compute [] :- wat.type/i64
  (wat.core/let
    [t (my.market/Tick :symbol "BTC" :price 50000.0 :volume 1000)
     v (my.market.Tick/volume t)]
    v))
