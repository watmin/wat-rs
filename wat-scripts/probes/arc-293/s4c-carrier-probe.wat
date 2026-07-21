(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure :Ok [value <- :wat::core::i64] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::i64::to-string
    (:wat::core::length (:my::Counter::surface-forms)))))
