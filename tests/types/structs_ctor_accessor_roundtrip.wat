;; structs_ctor_accessor_roundtrip.wat — user struct ctor + accessor round-trip.
(:wat::core::defstruct :my::market::Bar
  [open  <- :wat::core::f64
   close <- :wat::core::f64])
(:wat::core::defn :my::compute [] -> :wat::core::f64
  (:wat::core::let
    [b (:my::market::Bar :open 1.0 :close 2.0)
     o (:my::market::Bar/open b)
     c (:my::market::Bar/close b)]
    (:wat::f64::- c o)))
