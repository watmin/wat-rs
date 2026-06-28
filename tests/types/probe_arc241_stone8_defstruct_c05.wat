;; Contract 05: multiple fields, argspec stays rigid 3-slot triples.
(:wat::core::defstruct :my::Candle
  [open <- :wat::core::f64
   high <- :wat::core::f64
   low <- :wat::core::f64
   close <- :wat::core::f64
   volume <- :wat::core::i64])
