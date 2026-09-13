(:wat::core::defenum :u::E :wat::enum::Pure :A [x <- :wat::core::i64] :B)
(:wat::core::defenum :u::Box :wat::enum::Pure :Full [inner <- :u::E] :Empty)
(:wat::core::defn :u::g [b <- :u::Box] -> :wat::core::i64
  (:wat::core::match b
    [:u::Box.Full {:inner (:u::E.A x)} x]
    [_ 0]))
