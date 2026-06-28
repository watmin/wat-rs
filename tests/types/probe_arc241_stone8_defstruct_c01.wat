;; Contract 01: plain defstruct — no metadata.
(:wat::core::defstruct :my::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])
