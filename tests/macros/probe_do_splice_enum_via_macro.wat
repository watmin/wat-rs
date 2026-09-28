(:wat::core::defmacro :my::probe
  [body <- wat.type/AST]
  -> wat.type/AST
  `(:wat::core::do
     (:wat::core::defenum :my::probe::Event :wat::enum::Pure
       :Created [id <- wat.type/i64]
       :Deleted [id <- wat.type/i64]
       :NoOp)
     ~body))

(:my::probe
  (:wat::core::defn :my::probe::make-created [] -> :my::probe::Event (:my::probe::Event.Created {:id 1})))
