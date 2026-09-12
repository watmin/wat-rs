;; ConnectOutcome after the service has been stopped. Expected: Refused
;; (ECONNREFUSED / no listener). Prints the actual variant.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-connect-refused.wat

(:wat::core::defsurface :hold::Hold :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hold::Hold::PingRequest [])
   (:wat::core::defenum :hold::Hold::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :hold::Hold  req <- :hold::Hold::PingRequest] -> :hold::Hold::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :hold::hold
  :satisfies :hold::Hold
  :durable []
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::Ok)))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])])))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     addr (:hold::hold::Handle/addr h)
     _ (:wat::service::stop-faced (:hold::hold/stop h))
     o (:wat::kernel::connect addr)]
    (:wat::core::match o
      ((:wat::kernel::ConnectOutcome::Connected _)
        (:wat::kernel::println "connect-after-stop=Connected"))
      ((:wat::kernel::ConnectOutcome::Refused _)
        (:wat::kernel::println "connect-after-stop=Refused"))
      ((:wat::kernel::ConnectOutcome::Rejected _)
        (:wat::kernel::println "connect-after-stop=Rejected"))
      ((:wat::kernel::ConnectOutcome::Failed _)
        (:wat::kernel::println "connect-after-stop=Failed")))))
