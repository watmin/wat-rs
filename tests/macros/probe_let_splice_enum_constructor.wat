(:wat::core::let []
  (:wat::core::defenum :my::Request :wat::enum::Pure
    :Push [value <- :wat::core::i64]
    :NoOp)
  (:wat::core::defn :my::make-push [] -> :my::Request (:my::Request::Push 99)))
