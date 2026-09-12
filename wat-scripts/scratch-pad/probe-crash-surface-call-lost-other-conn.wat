;; CallOutcome::Lost — client B kills the service; client A (fresh connection,
;; never used) then call-by-deadline.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-call-lost-other-conn.wat

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
     (:wat::kernel::raise! (:wat::core::Fault/of "die-on-ping")))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     addr (:hold::hold::Handle/addr h)
     a (:wat::core::match (:wat::kernel::connect addr)
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: A dial failed" :wat::core::None :wat::core::None)))
     b (:wat::core::match (:wat::kernel::connect addr)
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: B dial failed" :wat::core::None :wat::core::None)))
     _ (:wat::core::match (:hold::Hold/ping b (:hold::Hold::PingRequest))
         (_ nil))
     inert (:hold::Hold::Reply::Ping (:hold::Hold::PingResponse::RequestTooLarge 0 0))
     o (:wat::service::call-by-deadline a
         (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)) 500 inert)]
    (:wat::core::match o
      ((:wat::service::CallOutcome::Answered _)
        (:wat::kernel::println "call-other-conn-after-die=Answered"))
      ((:wat::service::CallOutcome::DeadlineFired)
        (:wat::kernel::println "call-other-conn-after-die=DeadlineFired"))
      ((:wat::service::CallOutcome::Closed)
        (:wat::kernel::println "call-other-conn-after-die=Closed"))
      ((:wat::service::CallOutcome::Lost _)
        (:wat::kernel::println "call-other-conn-after-die=Lost"))
      ((:wat::service::CallOutcome::Malformed _)
        (:wat::kernel::println "call-other-conn-after-die=Malformed")))))
