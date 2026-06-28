;; newtype_construct_and_accessor_roundtrip.wat
(:wat::core::newtype :my::trading::Price :wat::core::f64)
(:wat::core::defn :my::compute [] -> :wat::core::String
  (:wat::core::let
    [p     (:my::trading::Price/new 100.0)
     inner (:my::trading::Price/0 p)]
    (:wat::core::f64::to-string inner)))
