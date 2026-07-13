(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s req] (:wat::service::Outcome::Reply s
                          (:probe::Echo::EchoResponse :reply (:probe::Echo::EchoRequest/msg req))))])
(:wat::core::defn :probe::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* n 2))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [nums (:wat::core::Vector :wat::core::i64 1 2 3 4 5)
     pr   (:wat::bracket::map (:wat::spawn::process) nums :probe::double)]
    (:wat::kernel::println (:wat::edn::write pr))))
