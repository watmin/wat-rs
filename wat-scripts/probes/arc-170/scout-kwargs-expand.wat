(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
             (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure :Ok [v <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::core::defn :probe::work
        [item <- :wat::core::String
         & [kv <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])]]
        -> :wat::core::String
        (:wat::core::match (:probe::Kv/get kv (:probe::Kv::GetRequest item)) ((:probe::Kv::GetResponse::Ok v) v)
  ((:probe::Kv::GetResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))
  ((:probe::Kv::GetResponse::RequestMalformed mpath mexpected mgot)
    (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))))))))
