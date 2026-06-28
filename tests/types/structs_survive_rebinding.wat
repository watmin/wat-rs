;; structs_survive_rebinding.wat — struct value survives let-rebinding + function call.
(:wat::core::defstruct :my::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])
(:wat::core::defn :my::y-of [p <- :my::Point] -> :wat::core::i64 (:my::Point/y p))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [p (:my::Point/new 3 7)
     q p]
    (:my::y-of q)))
