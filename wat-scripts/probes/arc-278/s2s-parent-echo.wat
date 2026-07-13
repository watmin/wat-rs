;; sanity: parent (echo's owner, in birth-seed as getppid) dials echo' — must work.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])
(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo :durable [] :ephemeral []
  :impls
  [(echo [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse :reply (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     c  (:wat::kernel::connect' (:probe::echo'::Handle/addr eh))
     r  (:probe::Echo/echo c (:probe::Echo::EchoRequest :msg "hi"))]
    (:wat::kernel::println (:probe::Echo::EchoResponse/reply r))))
