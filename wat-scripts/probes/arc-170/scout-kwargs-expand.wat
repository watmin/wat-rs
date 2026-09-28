(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Kv::GetRequest [k <- wat.type/String])
             (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure :Ok [v <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                     :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::core::write-forms
    (:wat::core::macroexpand (:wat::core::quote
      (:wat::core::defn :probe::work
        [item <- wat.type/String
         & [kv <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])]]
        -> wat.type/String
        (:wat::core::match (:probe::Kv/get kv (:probe::Kv::GetRequest item)) [:probe::Kv::GetResponse.Ok {:v v} v]
  [:probe::Kv::GetResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Kv::GetResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])))))))
