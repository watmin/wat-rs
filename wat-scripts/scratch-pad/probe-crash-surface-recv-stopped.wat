;; RecvOutcome::Stopped — park on recv, then the WORLD is asked to stop.
;; Prints READY so the outer command can SIGTERM the process.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-stopped.wat
;;   (from a wrapper that SIGTERMs after READY)
;;
;; Expected if the handler maps SIGTERM onto a parked recv: Stopped.
;; If the process dies without the recv returning, the wrapper observes no line
;; after READY — that is also a measurement.

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
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println "READY")
     o (:wat::kernel::recv p)
     _ (:wat::core::match o
         ((:wat::kernel::RecvOutcome::Message _)
           (:wat::kernel::println "parked-recv=Message"))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::println "parked-recv=Closed"))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::println "parked-recv=Stopped"))
         (:wat::kernel::RecvOutcome::TimedOut
           (:wat::kernel::println "parked-recv=TimedOut"))
         ((:wat::kernel::RecvOutcome::Lost _)
           (:wat::kernel::println "parked-recv=Lost"))
         ((:wat::kernel::RecvOutcome::Malformed _)
           (:wat::kernel::println "parked-recv=Malformed")))
     s (:wat::core::match
         (:wat::kernel::send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
         (:wat::kernel::SendOutcome::Sent "Sent")
         (:wat::kernel::SendOutcome::Closed "Closed")
         (:wat::kernel::SendOutcome::Stopped "Stopped")
         ((:wat::kernel::SendOutcome::Lost _) "Lost"))]
    (:wat::kernel::println
      (:wat::core::format "send-after-stopped={s}" :s s))))
