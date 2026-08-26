;; structs_user_method_auto_accessors.wat — user method uses auto-generated accessors.
(:wat::core::defstruct :my::market::Bar
  [high <- :wat::core::f64
   low  <- :wat::core::f64])
(:wat::core::defn :my::market::spread-of [b <- :my::market::Bar] -> :wat::core::f64 (:wat::f64::- (:my::market::Bar/high b) (:my::market::Bar/low b)))
(:wat::core::defn :my::compute [] -> :wat::core::f64
  (:wat::core::let
    [b (:my::market::Bar :high 10.0 :low 3.0)]
    (:my::market::spread-of b)))
