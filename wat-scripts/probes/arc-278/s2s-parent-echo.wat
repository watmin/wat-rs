;; sanity: parent (echo's owner, in birth-seed as getppid) dials echo' — must work.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                                      :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo :durable [] :ephemeral []
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse::Ok (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     c  (:wat::core::match (:wat::kernel::connect (:probe::echo::Handle/addr eh)) [:wat::kernel::ConnectOutcome::Connected {:peer p} p] [:wat::kernel::ConnectOutcome::Refused {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Rejected {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome::Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
     r  (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))]
    (:wat::kernel::println (:wat::core::match r [:wat::kernel::RecvOutcome::Message {:msg __recv} (:wat::core::match __recv 
  [:probe::Echo::EchoResponse::Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse::RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse::RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome::Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome::Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome::Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))))
