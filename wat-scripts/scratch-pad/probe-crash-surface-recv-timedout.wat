;; RecvOutcome::TimedOut — recv-by-deadline against a live silent client peer.
;; The hold service never sends unsolicited; a 300 ms deadline must fire.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-recv-timedout.wat

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
     lineage (:hold::hold::Handle/handle h)
     o (:wat::kernel::recv-by-deadline lineage 300)
     _ (:wat::core::match o
         (:wat::kernel::RecvOutcome::TimedOut
           (:wat::kernel::println "recv-by-deadline=TimedOut"))
         ((:wat::kernel::RecvOutcome::Message _)
           (:wat::kernel::println "recv-by-deadline=Message"))
         (:wat::kernel::RecvOutcome::Closed
           (:wat::kernel::println "recv-by-deadline=Closed"))
         (:wat::kernel::RecvOutcome::Stopped
           (:wat::kernel::println "recv-by-deadline=Stopped"))
         ((:wat::kernel::RecvOutcome::Lost _)
           (:wat::kernel::println "recv-by-deadline=Lost"))
         ((:wat::kernel::RecvOutcome::Malformed _)
           (:wat::kernel::println "recv-by-deadline=Malformed")))
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))
