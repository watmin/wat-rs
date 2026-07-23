;; sanity: parent (echo's owner, in birth-seed as getppid) dials echo' — must work.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo :durable [] :ephemeral []
  :impls
  [(echo [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse::Ok (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     c  (:wat::kernel::connect' (:probe::echo'::Handle/addr eh))
     r  (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))]
    (:wat::kernel::println (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
  ((:probe::Echo::EchoResponse::Ok reply) reply)
  ((:probe::Echo::EchoResponse::RequestTooLarge bytes cap)
    (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))
