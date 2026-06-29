;; ord_struct_bad.wat — Struct not in orderable class. Must FAIL.
(:wat::core::defstruct :my::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [p (:my::Point 1 2)
     q (:my::Point 3 4)]
    (:wat::core::< p q)))
