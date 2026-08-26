;; newtype_as_struct_field_roundtrip.wat
(:wat::core::newtype :my::trading::Price :wat::core::f64)
(:wat::core::defstruct :my::Order
  [label <- :wat::core::String
   price <- :my::trading::Price
   qty   <- :wat::core::i64])
(:wat::core::defn :my::compute [] -> :wat::core::String
  (:wat::core::let
    [p         (:my::trading::Price 99.5)
     o         (:my::Order :label "BTC" :price p :qty 7)
     retrieved (:my::Order/price o)
     inner     (:my::trading::Price/0 retrieved)]
    (:wat::f64::to-string inner)))
