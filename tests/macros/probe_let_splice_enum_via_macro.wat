(:wat::core::defmacro :my::probe
  [body <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::let []
     (:wat::core::defenum :my::probe::Event
       :Created [id <- :wat::core::i64]
       :Deleted [id <- :wat::core::i64]
       :NoOp)
     ~body))

(:my::probe
  (:wat::core::defn :my::probe::make-created [] -> :my::probe::Event (:my::probe::Event::Created 1)))
