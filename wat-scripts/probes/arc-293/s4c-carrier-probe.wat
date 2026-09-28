(:wat::core::defsurface :my::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :my::Counter::GetRequest  [])
   (:wat::core::defenum :my::Counter::GetResponse :wat::enum::Pure :Ok [value <- wat.type/i64] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                  :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features
  [(get [self <- :my::Counter  req <- :my::Counter::GetRequest] -> :my::Counter::GetResponse :max-request-bytes 524288)])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::i64::to-string
    (:wat::core::length (:my::Counter/surface-forms)))))
