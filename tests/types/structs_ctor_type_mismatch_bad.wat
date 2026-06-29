;; structs_ctor_type_mismatch_bad.wat — Bar/new open expects f64; pass String. Must FAIL.
(:wat::core::defstruct :my::market::Bar
  [open  <- :wat::core::f64
   close <- :wat::core::f64])
(:wat::core::defn :my::probe [] -> :my::market::Bar (:my::market::Bar "not-a-float" 2.0))
