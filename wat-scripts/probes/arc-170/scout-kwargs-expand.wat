(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Kv::GetReq [k <- :wat::core::String])
             (:wat::core::defenum :probe::Kv::GetResp :wat::enum::Pure :Ok [v <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetReq] -> :probe::Kv::GetResp :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::core::defn :probe::work
        [item <- :wat::core::String
         & [kv <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>]]
        -> :wat::core::String
        (:wat::core::match (:probe::Kv/get kv (:probe::Kv::GetReq item)) -> :wat::core::String
  ((:probe::Kv::GetResp::Ok v) v)
  ((:probe::Kv::GetResp::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None)))))))))
