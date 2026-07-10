(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defrecord :my::Counter::GetResponse [value <- :wat::core::i64])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::i64::to-string
    (:wat::core::length (:my::Counter::surface-forms)))))
