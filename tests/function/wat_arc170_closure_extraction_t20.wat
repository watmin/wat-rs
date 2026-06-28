;; T20: user enum Shape with tagged variants Rect/Circle — pattern bindings w,h,r must not be free.
(:wat::core::defenum :my::Shape
  :Rect [w <- :wat::core::i64
         h <- :wat::core::i64]
  :Circle [r <- :wat::core::i64])
(:wat::core::defn :my::shape-area [s <- :my::Shape] -> :wat::core::i64
  (:wat::core::match s -> :wat::core::i64
              ((:my::Shape::Rect w h) (:wat::core::i64::* w h))
              ((:my::Shape::Circle r) (:wat::core::i64::* r r))))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
