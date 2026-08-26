;; T20: user enum Shape with tagged variants Rect/Circle — pattern bindings w,h,r must not be free.
(:wat::core::defenum :my::Shape :wat::enum::Pure
  :Rect [w <- :wat::core::i64
         h <- :wat::core::i64]
  :Circle [r <- :wat::core::i64])
(:wat::core::defn :my::shape-area [s <- :my::Shape] -> :wat::core::i64
  (:wat::core::match s 
              ((:my::Shape::Rect w h) (:wat::i64::* w h))
              ((:my::Shape::Circle r) (:wat::i64::* r r))))
