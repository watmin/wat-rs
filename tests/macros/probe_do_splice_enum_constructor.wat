(:wat::core::do
  (:wat::core::defenum :my::Request
    :Push [value <- :wat::core::i64]
    :NoOp)
  (:wat::core::defn :my::make-push [] -> :my::Request (:my::Request::Push 99)))
