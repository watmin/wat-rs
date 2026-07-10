(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::Kv::GetReq [k <- :wat::core::String])
             (:wat::core::defrecord :probe::Kv::GetResp [v <- :wat::core::String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetReq] -> :probe::Kv::GetResp)])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::core::defn :probe::work
        [item <- :wat::core::String
         & [kv <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>]]
        -> :wat::core::String
        (:probe::Kv::GetResp/v (:probe::Kv/get kv (:probe::Kv::GetReq item)))))))))
