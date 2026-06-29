;; structs_ctor_arity_mismatch_bad.wat — Bar/new expects 2 args; pass 1. Must FAIL.
(:wat::core::defstruct :my::market::Bar
  [open  <- :wat::core::f64
   close <- :wat::core::f64])
(:wat::core::defn :my::probe [] -> :my::market::Bar (:my::market::Bar 1.0))
