;; structs_heterogeneous_fields.wat — struct with heterogeneous field types.
(:wat::core::defstruct :my::market::Tick
  [symbol <- :wat::core::String
   price  <- :wat::core::f64
   volume <- :wat::core::i64])
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [t (:my::market::Tick "BTC" 50000.0 1000)
     v (:my::market::Tick/volume t)]
    v))
