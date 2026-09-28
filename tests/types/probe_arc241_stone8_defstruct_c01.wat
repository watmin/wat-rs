;; Contract 01: plain defstruct — no metadata.
(:wat::core::defstruct :my::Point
  [x <- wat.type/i64
   y <- wat.type/i64])
