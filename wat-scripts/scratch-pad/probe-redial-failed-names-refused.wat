;; Drive :wat::service::redial-failed! on ConnectOutcome::Refused (reachable:
;; connect after stop). Rejected is UNREACHABLE from userland (crash-surface
;; matrix). Must still raise, and the message must name the SITE and REFUSED.
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-redial-failed-names-refused.wat

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
     _ (:wat::service::redial-failed! "probe: redial" (:wat::kernel::connect addr))]
    (:wat::kernel::println "UNREACHABLE: redial-failed! returned a peer")))
