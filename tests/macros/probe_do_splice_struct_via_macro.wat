(:wat::core::defmacro :my::probe
  [body <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do
     (:wat::core::defstruct :my::probe::Point
       [x <- :wat::core::i64
        y <- :wat::core::i64])
     ~body))

(:my::probe
  (:wat::core::defn :my::probe::make-origin [] -> :my::probe::Point (:my::probe::Point 0 0)))
