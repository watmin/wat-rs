;; newtype_rejected_where_inner_expected_bad.wat — Price where f64 expected. Must FAIL.
(:wat::core::newtype :my::trading::Price :wat::core::f64)
(:wat::core::defn :my::probe [] -> :wat::core::String
  (:wat::core::let
    [p     (:my::trading::Price/new 100.0)
     bogus (:wat::core::f64::+ p 1.0)]
    (:wat::core::f64::to-string bogus)))
